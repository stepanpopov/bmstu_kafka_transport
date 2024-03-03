use reqwest::{IntoUrl, Url};
use send_message::send_message;
use warp::{filters::BoxedFilter, http, Filter};

mod send_message;

pub trait ThreadSafeIntoUrl: IntoUrl + Clone + Send + Sync + 'static {}
impl<T: IntoUrl + Clone + Send + Sync + 'static> ThreadSafeIntoUrl for T {}

fn code_service_url_filter(
    code_service_url: impl ThreadSafeIntoUrl,
) -> BoxedFilter<(impl IntoUrl,)> {
    warp::any().map(move || code_service_url.clone()).boxed()
}

pub fn routes(
    code_service_url: impl ThreadSafeIntoUrl,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let send = warp::post()
        .and(warp::path("send"))
        .and(warp::path::end())
        .and(warp::body::json())
        .and(code_service_url_filter(code_service_url))
        .and_then(send_message);

    send
}
