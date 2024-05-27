use chrono::{DateTime, Duration, NaiveDate, Utc};
use common::SegmentWithTime;
use itertools::Itertools;
use log::{error, info, trace};
use std::{collections::HashMap, sync::RwLock, vec};

use super::consumer::Message;

pub struct MessageBuilder {
    retry_cache: SegmentsCache,
}

impl MessageBuilder {
    pub fn new(clean_interval: Duration, max_retry_num: u8) -> Self {
        Self {
            retry_cache: SegmentsCache::new(clean_interval, max_retry_num),
        }
    }

    pub fn build_messages(&self, got_segments: Vec<SegmentWithTime>) -> Vec<Message> {
        info!("cache state before: {:?}", &self.retry_cache);

        let messages = got_segments
            .into_iter()
            .group_by(|seg| seg.send_time.clone())
            .into_iter()
            .map(|(_, group)| group.collect::<Vec<SegmentWithTime>>())
            .flat_map(|segments| {
                let message = self.build_message(segments.clone());
                if message.is_none() {
                    self.retry_cache.add_bulk(segments.clone());
                }
                message
            })
            .collect::<Vec<Message>>();

        let messages_with_errors = self.build_messages_with_errors();

        info!("cache state after: {:?}", &self.retry_cache);

        self.retry_cache.increase_retry();
        self.retry_cache.clean_old_records();

        info!("messages: {:?}", messages);
        info!("messages with errors: {:?}", messages_with_errors);

        vec![messages, messages_with_errors]
            .into_iter()
            .flatten()
            .collect()
    }

    fn build_message(&self, got_segments_group: Vec<SegmentWithTime>) -> Option<Message> {
        let mut segments = got_segments_group;
        let first_segment = segments.first().unwrap();

        info!("Segments: {:?}", &segments);

        let send_time = first_segment.send_time.clone();
        if !self.retry_cache.check_retry_limit(&send_time) {
            info!("retry limit!");
            return None;
        }

        let sender = first_segment.segment.sender.clone();
        let segments_count = first_segment.segment.seg_count;

        let mut bitmap = vec![false; segments_count];
        for seg in segments.iter() {
            bitmap[seg.segment.seg_num] = true;
        }

        if !bitmap.iter().all(|b| *b) {
            info!("bitmap not full");
            let segments_from_cache = match self.retry_cache.get_segments(&send_time, &bitmap) {
                Some(s) => s,
                None => return None,
            };
            info!("segments from cache: {:?}", segments_from_cache);

            segments.extend(segments_from_cache)
        } else {
            info!("bitmap full");
        }

        segments.sort_unstable_by_key(|seg| seg.segment.seg_num);

        let full_payload = segments.into_iter().map(|seg| seg.segment.payload).concat();
        let full_payload_u16: Vec<u16> = full_payload
            .chunks(2)
            .into_iter()
            .map(|chunk| u16::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        let string_payload = String::from_utf16(&full_payload_u16).unwrap();

        Some(Message {
            payload: string_payload,
            has_error: false,
            sender,
            send_time,
        })
    }

    fn build_messages_with_errors(&self) -> Vec<Message> {
        self.retry_cache
            .get_latest_invalidated_records()
            .into_iter()
            .map(|r| Message {
                payload: "".to_string(),
                has_error: true,
                sender: r.sender,
                send_time: r.send_time,
            })
            .collect()
    }
}

#[derive(Clone, Debug)]
struct SegmentsCacheRecord {
    segments: Vec<SegmentWithTime>,
    num_bit_map: Vec<bool>,
    retry: u8,
}

#[derive(Clone)]
struct InvalidatedRecord {
    sender: String,
    send_time: String,
}

#[derive(Debug)]
struct SegmentsCache {
    cache: RwLock<HashMap<DateTime<Utc>, SegmentsCacheRecord>>,
    clean_interval: Duration,
    max_retry_num: u8,
}

impl SegmentsCache {
    pub fn new(clean_interval: Duration, max_retry_num: u8) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            clean_interval,
            max_retry_num,
        }
    }

    pub fn increase_retry(&self) {
        for (_k, v) in self.write_to_cache().iter_mut() {
            if v.retry < self.max_retry_num {
                v.retry += 1;
            }
        }
    }

    pub fn check_retry_limit(&self, send_time: &str) -> bool {
        let key = Self::get_key_from_send_time(send_time);

        match self.read_cache().get(&key) {
            Some(r) => r.retry < self.max_retry_num,
            None => true,
        }
    }

    pub fn get_latest_invalidated_records(&self) -> Vec<InvalidatedRecord> {
        self.read_cache()
            .iter()
            .filter_map(|(_, r)| {
                let first_segment = match r.segments.get(0) {
                    Some(s) => s,
                    None => return None,
                };

                if r.retry == self.max_retry_num - 1 {
                    return Some(InvalidatedRecord {
                        sender: first_segment.segment.sender.clone(),
                        send_time: first_segment.send_time.clone(),
                    });
                }

                None
            })
            .collect()
    }

    // to use after check got_bit_map for all true
    pub fn get_segments(
        &self,
        send_time: &str,
        got_bit_map: &[bool],
    ) -> Option<Vec<SegmentWithTime>> {
        let key = Self::get_key_from_send_time(send_time);
        info!("get segments, key: {:?}", key);

        let record = match self.read_cache().get(&key) {
            Some(r) => r.clone(),
            None => return None,
        };

        info!("get segments, record: {:?}", record);

        if record.num_bit_map.len() != got_bit_map.len() {
            info!("bit_map wrong size");
            panic!("bit_map wrong size");
        }

        let mut bitmap_all_true = true;

        for (ind, stored) in record.num_bit_map.iter().enumerate() {
            if !stored && !got_bit_map[ind] {
                bitmap_all_true = false;
                break;
            }
        }

        info!("get segments, bitmap_all_true: {}", bitmap_all_true);

        if !bitmap_all_true {
            return None;
        }

        let stored_segments = &record.segments;

        let segments: Vec<SegmentWithTime> = record
            .num_bit_map
            .iter()
            .zip(got_bit_map)
            .enumerate()
            .filter_map(|(ind, (stored, got))| {
                if *stored && !got {
                    let stored_segment = stored_segments
                        .iter()
                        .find(|s| s.segment.seg_num == ind)
                        .unwrap();

                    Some(stored_segment.clone())
                } else {
                    None
                }
            })
            .collect();

        Some(segments)
    }

    pub fn add_bulk(&self, segmets: Vec<SegmentWithTime>) {
        for s in segmets {
            self.add(s);
        }
    }

    pub fn add(&self, seg: SegmentWithTime) {
        let key = Self::get_key_from_send_time(&seg.send_time);

        let seg_num = seg.segment.seg_num;
        let seg_count = seg.segment.seg_count;

        let record_to_insert = match self.write_to_cache().get_mut(&key) {
            Some(rec) => {
                rec.segments.push(seg);
                rec.num_bit_map[seg_num] = true;
                None
            }
            None => {
                let mut num_bit_map = vec![false; seg_count];
                num_bit_map[seg_num] = true;

                let segments = vec![seg];

                let record = SegmentsCacheRecord {
                    segments,
                    num_bit_map,
                    retry: 0,
                };

                Some(record)
                //self.write_to_cache().insert(key, record);
            }
        };

        match record_to_insert {
            Some(r) => {
                self.write_to_cache().insert(key, r);
            }
            None => return,
        }
    }

    pub fn remove(&self, key: &DateTime<Utc>) -> bool {
        self.write_to_cache().remove(key).is_some()
    }

    pub fn clean_old_records(&self) {
        let now = Utc::now();

        self.write_to_cache()
            .retain(|key, record| (now - *key < self.clean_interval));

        for (_k, v) in self.write_to_cache().iter_mut() {
            if v.retry == self.max_retry_num {
                v.segments.clear();
                v.num_bit_map.clear();
            }
        }
    }

    fn read_cache(
        &self,
    ) -> std::sync::RwLockReadGuard<'_, HashMap<DateTime<Utc>, SegmentsCacheRecord>> {
        self.cache.read().unwrap()
    }

    fn write_to_cache(
        &self,
    ) -> std::sync::RwLockWriteGuard<'_, HashMap<DateTime<Utc>, SegmentsCacheRecord>> {
        self.cache.write().unwrap()
    }

    fn get_key_from_send_time(send_time: &str) -> DateTime<Utc> {
        let timestamp_key: usize = send_time.parse().unwrap();
        let key: DateTime<Utc> = DateTime::from_timestamp_millis(timestamp_key as i64).unwrap();

        key
    }
}
