use crate::app::{CustomError, Serialize};
use warp::{reply, Reply, Filter, reject, Rejection, http::StatusCode};

#[derive(Serialize)]
struct ErrorMessage {
    code: u16,
    message: String,
}

pub(crate) async fn handle_rejection(err: Rejection) -> Result<impl Reply, Rejection> {
    let code;
    let message;

    if err.is_not_found() {
        message = "Not Found".to_string();
        code = StatusCode::NOT_FOUND;
        eprintln!("1");
    } else if let Some(_) = err.find::<warp::filters::body::BodyDeserializeError>() {
        message = "Invalid Body".to_string();
        code = StatusCode::BAD_REQUEST;
        eprintln!("2");
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        message = "Method Not Allowed".to_string();
        code = StatusCode::METHOD_NOT_ALLOWED;
        eprintln!("3");
    } else if let Some(e) = err.find::<CustomError>() {
        message = e.to_string();
        code = match &e {
            CustomError::MissingParameters => StatusCode::BAD_REQUEST,
            CustomError::QuestionNotFound => StatusCode::NOT_FOUND,
            CustomError::ChannelNotFound => StatusCode::NOT_FOUND,
            CustomError::ChannelNotPresence => StatusCode::BAD_REQUEST,
            CustomError::ChannelsNotFound => StatusCode::NOT_FOUND,
            CustomError::EventChannelEmpty => StatusCode::NOT_FOUND,
            CustomError::NotFound => StatusCode::NOT_FOUND,
            CustomError::AppKeyNotFound => StatusCode::NOT_FOUND,
            CustomError::AppIdNotFound => StatusCode::NOT_FOUND,
            CustomError::AuthKeyMismatch => StatusCode::UNAUTHORIZED,
            CustomError::AuthSignatureError => StatusCode::UNAUTHORIZED,
        };
        eprintln!("4");
    } else {
        eprintln!("unhandled error: {:?}", err);
        message = "Internal Server Error".to_string();
        code = StatusCode::INTERNAL_SERVER_ERROR;
        eprintln!("5");
    }

    let json = warp::reply::json(&ErrorMessage {
        code: code.as_u16(),
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}