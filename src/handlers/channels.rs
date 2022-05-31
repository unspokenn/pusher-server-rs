use crate::app::{PusherQuery, Pusher, JsonResponse};
use crate::handlers::{ChannelResponse, ChannelsResponse};

pub(crate) async fn list_channels(pusher: Pusher, query: PusherQuery) -> JsonResponse {
    let response = ChannelsResponse::from((pusher.get_channels().await?, &query));

    Ok(warp::reply::json(&response))
}

pub(crate) async fn get_channel(pusher: Pusher, query: PusherQuery, channel_name: String) -> JsonResponse {
    let response = ChannelResponse::from((&pusher.get_channel(channel_name).await?, &query));

    Ok(warp::reply::json(&response))
}
