use std::sync::Arc;

use warp::{http, reply::Reply, Filter};

use common::SegmentWithTime;

use crate::producer::SegmentProducer;

pub async fn produce_segments(
    segment: SegmentWithTime,
    producer: Arc<SegmentProducer>,
    topic_name: String,
) -> Result<warp::reply::Response, warp::Rejection> {
    match producer.produce_segment(&topic_name, segment).await {
        Ok(_) => {
            return Ok(
                warp::reply::with_status("Segment sent", http::StatusCode::OK).into_response(),
            )
        }
        Err(e) => {
            return Ok(warp::reply::with_status(
                format!("Failed to send segment: {}", e),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    }
}
