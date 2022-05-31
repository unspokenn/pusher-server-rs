mod errors;
mod channels;
mod events;
mod websocket;
mod responses;

pub(crate) use errors::handle_rejection;
pub(crate) use responses::{ChannelsResponse, ChannelResponse};
pub(crate) use channels::{get_channel, list_channels};
pub(crate) use events::event_create;
pub(crate) use websocket::ws;

pub(crate) async fn index() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::http::status::StatusCode::NOT_FOUND)
}

pub(crate) async fn health() -> Result<impl warp::Reply, warp::Rejection> {
    Ok(warp::http::status::StatusCode::OK)
}
