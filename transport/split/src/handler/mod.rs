use warp::{http, Filter};
use send_message::send_message;

mod send_message;

pun fn routes() {
    let send = warp::post()
        .and(warp::path("send"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and_then(send_message);

    add_items
}
