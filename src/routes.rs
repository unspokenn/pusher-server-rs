use warp::{Filter, Rejection};
use warp::filters::BoxedFilter;
use warp::hyper::{Method, StatusCode};
use warp::path::FullPath;
use crate::app::{PusherQuery, PusherServer, Pusher, CustomError, Channel, EventRequestBody};

use crate::handlers;

pub(crate) fn routes(server: PusherServer, app_name: &'static str) -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    index_filter()
        .or(health_filter())
        .or(event_filter(&server).and_then(handlers::event_create).recover(handlers::handle_rejection))
        .or(channel_filter(&server).and_then(handlers::get_channel).recover(handlers::handle_rejection))
        .or(channels_filter(&server).and_then(handlers::list_channels).recover(handlers::handle_rejection))
        .or(websocket_filter(&server).and_then(handlers::ws).recover(handlers::handle_rejection))
        .with(warp::cors().allow_any_origin())
        .with(warp::log(app_name))
}

#[inline(always)]
pub(crate) fn index_filter() -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    warp::path::end().and_then(handlers::index)
}

#[inline(always)]
pub(crate) fn websocket_filter(server: &PusherServer) -> BoxedFilter<(Pusher, warp::filters::ws::Ws)> {
    validate_app_by_key(server).and(warp::ws()).boxed()
}

#[inline(always)]
pub(crate) fn event_filter(server: &PusherServer) -> BoxedFilter<(Pusher, PusherQuery, EventRequestBody)> {
    validate_app_by_id(server).and(warp::path!("events")).and(warp::path::end()).and(warp::post()).and(json_body()).boxed()
}

#[inline(always)]
pub(crate) fn channels_filter(server: &PusherServer) -> BoxedFilter<(Pusher, PusherQuery)> {
    validate_app_by_id(server).and(warp::path!("channels")).and(warp::path::end()).and(warp::get()).boxed()
}

#[inline(always)]
pub(crate) fn channel_filter(server: &PusherServer) -> BoxedFilter<(Pusher, PusherQuery, String)> {
    validate_app_by_id(server).and(warp::path!("channels" / String)).and(warp::path::end()).and(warp::get()).boxed()
}

#[inline(always)]
pub(crate) fn health_filter() -> impl Filter<Extract = impl warp::Reply, Error = Rejection> + Clone {
    eprintln!("{:#?}", "14");
    warp::path!("health").and(warp::get()).and_then(handlers::health)
}


#[inline(always)]
pub(crate) fn with_pusher_server(server: PusherServer) -> impl Filter<Extract = (PusherServer, ), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || server.clone())
}

#[inline(always)]
pub(crate) fn json_body() -> impl Filter<Extract = (EventRequestBody, ), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[inline(always)]
pub(crate) fn path() -> impl Filter<Extract = (String, ), Error = std::convert::Infallible> + Clone {
    warp::path::full().and(warp::method()).map(|path: FullPath, method: Method| format!("{}\n{}\n", method, path.as_str()))
}

#[inline(always)]
fn validate_app_by_key(server: &PusherServer) -> impl Filter<Extract = (Pusher, ), Error = Rejection> + Clone {
    warp::path!("app" / String)
        .and(warp::query::<PusherQuery>())
        .and(with_pusher_server(server.clone()))
        .and(path())
        .and_then(|app_key: String, query: PusherQuery, server: PusherServer, path: String| async move {
            match server.find(app_key.as_str()).unwrap_or_default().ensure_valid_signature(&query, path.as_str()) {
                Ok(pusher) => Ok(pusher),
                Err(e) => Err(e)
            }
        })
}

#[inline(always)]
fn validate_app_by_id(server: &PusherServer) -> impl Filter<Extract = (Pusher, PusherQuery, ), Error = Rejection> + Clone {
    warp::path!("apps" / u32 / .. )
        .and(warp::query::<PusherQuery>())
        .and(with_pusher_server(server.clone()))
        .and(path())
        .and_then(|app_id: u32, query: PusherQuery, server: PusherServer, path: String| async move {
            match server.find_by_id(app_id).unwrap_or_default().ensure_valid_signature(&query, path.as_str()) {
                Ok(pusher) => Ok((pusher, query)),
                Err(e) => Err(e)
            }
        }).untuple_one()
}
