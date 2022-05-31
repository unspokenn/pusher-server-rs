use crate::app::{EventRequestBody, JsonResponse, Pusher, PusherQuery, json};

pub(crate) async fn event_create(pusher: Pusher, _query: PusherQuery, request: EventRequestBody) -> JsonResponse {
    request.payload_publish(pusher).await?;
    Ok(warp::reply::json(&json!({})))
}


