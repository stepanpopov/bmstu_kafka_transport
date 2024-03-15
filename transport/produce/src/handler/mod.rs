use std::sync::Arc;

use produce_segments::produce_segments;
use warp::{filters::BoxedFilter, Filter};

use crate::producer::SegmentProducer;

mod produce_segments;

fn producer_filter(producer: Arc<SegmentProducer>) -> BoxedFilter<(Arc<SegmentProducer>,)> {
    warp::any().map(move || producer.clone()).boxed()
}

fn topic_name_filter(topic_name: String) -> BoxedFilter<(String,)> {
    warp::any().map(move || topic_name.clone()).boxed()
}

pub fn routes(
    producer: Arc<SegmentProducer>,
    topic_name: String,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    

    warp::post()
        .and(warp::path("transfer"))
        .and(warp::body::json())
        .and(producer_filter(producer))
        .and(topic_name_filter(topic_name))
        .and_then(produce_segments)
}
