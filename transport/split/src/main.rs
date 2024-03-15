use std::net::SocketAddr;
use warp::{http, Filter};

use config::Config;
use handler::routes;

use log::{error, info};

use common::setup_env_logger;

mod config;
mod handler;

#[tokio::main]
async fn main() {
    // let config = Config::from_env();
    let config = Config::build().cmd_parse().get_env();

    setup_env_logger(true, "RUST_LOG");

    warp::serve(routes(config.code_service_url, config.chunk_byte_size))
        .run(config.listen.parse::<SocketAddr>().unwrap())
        .await;
}
