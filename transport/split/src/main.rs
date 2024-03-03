use std::net::SocketAddr;
use warp::{http, Filter};

use config::Config;
use handler::routes;

mod config;
mod handler;

#[tokio::main]
async fn main() {
    // let config = Config::from_env();
    let config = Config::build().cmd_parse().get_env();

    // let url = "http://localhost:8080/code";

    warp::serve(routes(config.code_service_url))
        .run(config.listen.parse::<SocketAddr>().unwrap())
        .await;
}
