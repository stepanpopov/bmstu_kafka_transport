use log::info;

use chrono::{DateTime, Duration, NaiveDate, Utc};

use rdkafka::client::ClientContext;
use rdkafka::consumer::{ConsumerContext, Rebalance};
use rdkafka::error::KafkaResult;
use rdkafka::topic_partition_list::TopicPartitionList;
use rdkafka::util::get_rdkafka_version;

use common::setup_env_logger;

mod command;
mod consumer;
mod message_builder;
mod sender;

use command::Config;
use consumer::SegmentConsumer;
use sender::MessageSender;

use crate::message_builder::MessageBuilder;

// A context can be used to change the behavior of producers and consumers by adding callbacks
// that will be executed by librdkafka.
// This particular context sets up custom callbacks to log rebalancing events.

// A type alias with your custom consumer can be created for convenience.
// type LoggingConsumer = BaseConsumer<CustomContext>;

pub struct SegmentConsumerContext;

impl ClientContext for SegmentConsumerContext {}

impl ConsumerContext for SegmentConsumerContext {
    fn pre_rebalance(&self, rebalance: &Rebalance) {
        info!("Pre rebalance {:?}", rebalance);
    }

    fn post_rebalance(&self, rebalance: &Rebalance) {
        info!("Post rebalance {:?}", rebalance);
    }

    fn commit_callback(&self, result: KafkaResult<()>, _offsets: &TopicPartitionList) {
        info!("Committing offsets: {:?}", result);
    }
}

#[tokio::main]
async fn main() {
    let config = Config::from_cmd();
    setup_env_logger(true, "RUST_LOG");

    let (version_n, version_s) = get_rdkafka_version();
    info!("rd_kafka_version: 0x{:08x}, {}", version_n, version_s);

    let message_sender = MessageSender::new(config.receive_url).unwrap();
    let message_builder = MessageBuilder::new(Duration::minutes(30), 3);

    let mut consumer =
        SegmentConsumer::new(SegmentConsumerContext, &config.group_id, &config.brokers);

    consumer.subscribe(&config.topic);

    let _ = consumer
        .start_consume_and_send(
            message_sender,
            message_builder,
            tokio::time::Duration::from_secs(30),
        )
        .await;
}
