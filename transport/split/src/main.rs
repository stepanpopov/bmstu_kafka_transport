use warp::{http, Filter};
use std::net::SocketAddr;


use handler::routes;
use config::Config;

mod config;
mod handler;


#[tokio::main]
async fn main() {

    let config = Config::from_env();

    warp::serve(routes)
        .run(config.listen.parse::<SocketAddr>().unwrap())
        .await;
}
