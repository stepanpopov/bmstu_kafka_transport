use log::{error, info, trace};
use serde::{Deserialize, Serialize};

use tokio::time::Duration;

use tokio_stream::StreamExt;

use anyhow::{anyhow, Error};
use itertools::Itertools;

use std::sync::Arc;
use std::time;

use rdkafka::client::ClientContext;
use rdkafka::config::{ClientConfig, RDKafkaLogLevel};

use rdkafka::consumer::{BaseConsumer, Consumer, ConsumerContext};
use rdkafka::message::Message as KafkaMessage;

use crate::message_builder::{self, MessageBuilder};

use super::sender::MessageSender;

use common::SegmentWithTime;

#[derive(Deserialize, Serialize, Debug)]
pub struct Message {
    pub payload: String,
    pub has_error: bool,
    pub send_time: String,
    pub sender: String,
}

pub struct SegmentConsumer<T: ClientContext + ConsumerContext> {
    base: BaseConsumer<T>,
    topic: Option<String>,
}

impl<T: ClientContext + ConsumerContext> SegmentConsumer<T> {
    pub fn new(context: T, group_id: &str, brokers: &str) -> Self {
        let consumer = ClientConfig::new()
            .set("group.id", group_id)
            .set("bootstrap.servers", brokers)
            .set("enable.partition.eof", "false")
            .set("session.timeout.ms", "6000")
            .set("enable.auto.commit", "true")
            //.set("statistics.interval.ms", "30000")
            //.set("auto.offset.reset", "smallest")
            .set_log_level(RDKafkaLogLevel::Debug)
            .create_with_context(context)
            .expect("Consumer creation failed");

        Self {
            base: consumer,
            topic: None,
        }
    }

    pub fn subscribe(&mut self, topic: &str) {
        self.base
            .subscribe(&[topic])
            .expect("Can't subscribe to specified topic");

        self.topic = Some(topic.into());
    }

    pub async fn start_consume_and_send(
        &self,
        sender: MessageSender,
        message_builder: MessageBuilder,
        collect_message_interval: Duration,
    ) -> Result<(), Error> {
        if self.topic.is_none() {
            return Err(anyhow!("call subscribe first"));
        }

        info!("Started to consume and send");

        // self.base.

        let sender = Arc::new(sender);
        let message_builder = Arc::new(message_builder);

        //let mut interval_stream = IntervalStream::new(interval);

        loop {
            let mut segments: Vec<SegmentWithTime> = vec![];

            let start_time = time::Instant::now();

            loop {
                if start_time.elapsed() > collect_message_interval {
                    trace!("Stopping loop after: {:?}", start_time.elapsed());
                    break;
                }

                // Detach can be done before moving to another thread and building segment there to perf upupup
                let res = match self.base.poll(Duration::from_millis(500)) {
                    Some(mess) => match mess {
                        Ok(mess) => mess.detach(),
                        Err(e) => {
                            error!("{}", e);
                            continue;
                        }
                    },
                    None => {
                        continue;
                    }
                };

                info!("got message: {:?}", &res);

                let segment = res.into();

                segments.push(segment);
            }

            //if !segments.is_empty() {
            let sender = sender.clone();
            let message_builder = message_builder.clone();

            tokio::spawn(async move {
                let messages =
                    tokio::task::spawn_blocking(move || message_builder.build_messages(segments))
                        .await
                        .unwrap();

                for mes in messages {
                    sender.send_message(&mes).await.unwrap();
                }
            });
            //}
        }

        Ok(())
    }

    pub fn consumer(&self) -> &BaseConsumer<T> {
        &self.base
    }

    pub fn get_all_partitions(&self, topic: &str, fetch_timeout: Duration) -> Vec<i32> {
        self.base
            .fetch_metadata(Some(topic), fetch_timeout)
            .unwrap()
            .topics()
            .iter()
            .find(|t| t.name() == topic)
            .unwrap()
            .partitions()
            .iter()
            .map(|p| p.id())
            .collect()
    }
}

/*loop {
    match consumer.recv().await {
        Err(e) => warn!("Kafka error: {}", e),
        Ok(m) => {
            let payload = match m.payload_view::<str>() {
                None => "",
                Some(Ok(s)) => s,
                Some(Err(e)) => {
                    warn!("Error while deserializing message payload: {:?}", e);
                    ""
                }
            };
            info!("key: '{:?}', payload: '{}', topic: {}, partition: {}, offset: {}, timestamp: {:?}",
                    m.key(), payload, m.topic(), m.partition(), m.offset(), m.timestamp());
            if let Some(headers) = m.headers() {
                for header in headers.iter() {
                    info!("  Header {:#?}: {:?}", header.key, header.value);
                }
            }
            consumer.commit_message(&m, CommitMode::Async).unwrap();
        }
    };
}*/
