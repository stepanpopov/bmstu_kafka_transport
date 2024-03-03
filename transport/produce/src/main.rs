use producer::SegmentProducer;
use std::{net::SocketAddr, sync::Arc};
use warp::{http, Filter};

use config::Config;
use handler::routes;

mod config;
mod handler;
mod producer;

#[tokio::main]
async fn main() {
    let config = Config::build().cmd_parse().get_env();

    let producer = SegmentProducer::new(&config.brokers);
    let producer = Arc::new(producer);

    warp::serve(routes(producer, config.topic))
        .run(config.listen.parse::<SocketAddr>().unwrap())
        .await;
}
