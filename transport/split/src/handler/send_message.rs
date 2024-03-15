

use reqwest::{self, IntoUrl};
use serde::{Deserialize, Serialize};
use warp::{http, reply::Reply, Filter};

use common::{Segment, SegmentWithTime};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message {
    sender: String,
    time: String,
    payload: Vec<u8>,
}

fn split_message(m: Message, chunk_byte_size: usize) -> Vec<SegmentWithTime> {
    let splitted_payload = m.payload.chunks(chunk_byte_size);
    let seg_count = splitted_payload.len();

    let mut i = 0;
    splitted_payload
        .map(|c| {
            let segment = Segment {
                sender: m.sender.clone(),
                seg_count,
                payload: c.to_vec(),
                seg_num: i,
            };
            i += 1;

            SegmentWithTime {
                segment,
                send_time: m.time.clone(),
            }
        })
        .collect()
}

pub async fn send_message(
    m: Message,
    code_service_url: impl IntoUrl,
    chunk_byte_size: usize,
) -> Result<warp::reply::Response, warp::Rejection> {
    let segments = split_message(m, chunk_byte_size);

    let client = reqwest::Client::new();

    let url = code_service_url.into_url().unwrap();

    for segment in segments {
        let resp = match client.post(url.clone()).json(&segment).send().await {
            Ok(resp) => resp,
            Err(e) => {
                return Ok(warp::reply::with_status(
                    format!("Failed to send segment: {}", e),
                    http::StatusCode::INTERNAL_SERVER_ERROR,
                )
                .into_response());
            }
        };

        if !resp.status().is_success() {
            return Ok(warp::reply::with_status(
                "Failed to send segment",
                http::StatusCode::INTERNAL_SERVER_ERROR,
            )
            .into_response());
        }
    }

    Ok(
        warp::reply::with_status("Segments sended successfully", http::StatusCode::OK)
            .into_response(),
    )
}
