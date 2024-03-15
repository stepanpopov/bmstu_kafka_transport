use std::time::Duration;

use anyhow::{anyhow, Result};


use rdkafka::producer::{FutureProducer};
use rdkafka::ClientConfig;

use common::SegmentWithTime;

pub struct SegmentProducer {
    base: FutureProducer,
}

impl SegmentProducer {
    pub fn new(brokers: &str) -> Self {
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .expect("Producer creation error");

        Self { base: producer }
    }

    pub async fn produce_segment(&self, topic_name: &str, segment: SegmentWithTime) -> Result<()> {
        self.base
            .send(
                segment.into_future_record(topic_name),
                Duration::from_secs(0),
            )
            .await
            .map_err(|e| anyhow!(e.0))?;

        Ok(())
    }
}
