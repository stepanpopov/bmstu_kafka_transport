use log::info;
use producer::SegmentProducer;
use std::{net::SocketAddr, sync::Arc};
use warp::{http, Filter};

use config::Config;
use handler::routes;

mod config;
mod handler;
mod producer;

use common::setup_env_logger;

#[tokio::main]
async fn main() {
    setup_env_logger(true, "RUST_LOG");

    let config = Config::build().cmd_parse().get_env();

    info!("Config: {:?}", config);

    let producer = SegmentProducer::new(&config.brokers);
    let producer = Arc::new(producer);

    warp::serve(routes(producer, config.topic))
        .run(config.listen.parse::<SocketAddr>().unwrap())
        .await;
}
