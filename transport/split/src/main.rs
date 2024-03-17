use std::net::SocketAddr;

use config::Config;
use handler::routes;

use common::setup_env_logger;
use log::info;

mod config;
mod handler;

#[tokio::main]
async fn main() {
    // let config = Config::from_env();
    let config = Config::build().cmd_parse().get_env();

    setup_env_logger(true, "RUST_LOG");

    info!("Config: {:?}", config);

    warp::serve(routes(config.code_service_url, config.chunk_byte_size))
        .run(config.listen.parse::<SocketAddr>().unwrap())
        .await;
}
