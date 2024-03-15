use std::thread;
use std::{io::Write};

use rdkafka::message::{Headers, OwnedHeaders, OwnedMessage};
use rdkafka::producer::FutureRecord;
use rdkafka::Message;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use chrono::prelude::*;
use env_logger::fmt::Formatter;
use env_logger::Builder;
use log::{LevelFilter, Record};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Segment {
    pub payload: Vec<u8>,
    // pub send_time: String, // id
    pub seg_count: usize,
    pub seg_num: usize,
    pub sender: String,
}

#[derive(Debug)]
pub struct SegmentWithTime {
    pub segment: Segment,
    pub send_time: String,
}

impl SegmentWithTime {
    pub fn into_future_record<'a>(
        &'a self,
        topic_name: &'a str,
    ) -> FutureRecord<'_, String, Vec<u8>> {
        FutureRecord::to(topic_name)
            .payload(&self.segment.payload)
            .key(&self.send_time)
            .headers(self.segment.clone().into())
    }
}

impl From<Segment> for OwnedHeaders {
    fn from(val: Segment) -> Self {
        OwnedHeaders::new()
            .add("seg_count", &val.seg_count.to_ne_bytes())
            .add("seg_num", &val.seg_num.to_ne_bytes())
            .add("sender", &val.sender)
    }
}

impl From<OwnedMessage> for SegmentWithTime {
    fn from(value: OwnedMessage) -> Self {
        let send_time: String = String::from_utf8(value.key().unwrap().to_vec()).unwrap();

        let headers = value.headers().unwrap();

        let seg_count = usize::from_ne_bytes(
            headers
                .get(0)
                .unwrap()
                .1
                .try_into()
                .expect("Incorrect byte slice length"),
        );
        let seg_num = usize::from_ne_bytes(
            headers
                .get(1)
                .unwrap()
                .1
                .try_into()
                .expect("Incorrect byte slice length"),
        );
        let sender = String::from_utf8(headers.get(2).unwrap().1.to_vec()).unwrap();

        let payload = value.payload().unwrap().to_vec();

        let segment = Segment {
            payload,
            seg_count,
            seg_num,
            sender,
        };

        Self { send_time, segment }
    }
}

// TODO
// maybe there are more gentle ways......

// impl trait to serialize objects with plain struct to SegmentWithTime with nested segment
impl Serialize for SegmentWithTime {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("SegmentWithTime", 5)?;
        s.serialize_field("send_time", &self.send_time)?;

        s.serialize_field("payload", &self.segment.payload)?;
        s.serialize_field("seg_count", &self.segment.seg_count)?;
        s.serialize_field("seg_num", &self.segment.seg_num)?;
        s.serialize_field("sender", &self.segment.sender)?;

        s.end()
    }
}

// impl trait to deserialize objects with plain struct to SegmentWithTime with nested segment
impl<'de> Deserialize<'de> for SegmentWithTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Serialize)]
        struct _SegmentWithTime {
            pub payload: Vec<u8>,
            pub send_time: String,
            pub seg_count: usize,
            pub seg_num: usize,
            pub sender: String,
        }

        let got = _SegmentWithTime::deserialize(deserializer)?;

        let segment = Segment {
            payload: got.payload,
            seg_count: got.seg_count,
            seg_num: got.seg_num,
            sender: got.sender,
        };

        Ok(SegmentWithTime {
            send_time: got.send_time,
            segment,
        })
    }
}

pub fn setup_env_logger(log_thread: bool, default_env_var_log_level: &str) {
    let output_format = move |formatter: &mut Formatter, record: &Record| {
        let thread_name = if log_thread {
            format!("(t: {}) ", thread::current().name().unwrap_or("unknown"))
        } else {
            "".to_string()
        };

        let local_time: DateTime<Local> = Local::now();
        let time_str = local_time.format("%H:%M:%S%.3f").to_string();
        writeln!(
            formatter,
            "{} {}{} - {} - {}",
            time_str,
            thread_name,
            record.level(),
            record.target(),
            record.args()
        )
    };

    let mut builder = Builder::new();
    builder.format(output_format);

    if std::env::var(default_env_var_log_level).is_err() {
        builder.filter_level(LevelFilter::Info);
    }

    // rust_log.map(|conf| builder.parse_filters(conf));

    builder.init();
}
