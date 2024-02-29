use serde::{Deserialize, Serialize};
use warp::{http, Filter};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Message {
    sender: String,
    time: String,
    payload: String,
}

async fn send_message(m: Message) -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::reply::with_status(
        "Message sended successfully",
        http::StatusCode::CREATED,
    ))
}
