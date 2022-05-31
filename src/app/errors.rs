#[derive(Debug)]
pub(crate) enum CustomError {
    MissingParameters,
    QuestionNotFound,
    ChannelNotFound,
    ChannelNotPresence,
    ChannelsNotFound,
    EventChannelEmpty,
    NotFound,
    AppKeyNotFound,
    AppIdNotFound,
    AuthKeyMismatch,
    AuthSignatureError,
}

impl warp::reject::Reject for CustomError {}

impl std::fmt::Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match *self {
            // Error::ParseError(ref err) => write!(f, "Cannot parse parameter: {}", err),
            CustomError::MissingParameters => write!(f, "Missing parameter"),
            CustomError::QuestionNotFound => write!(f, "Question not found"),
            CustomError::ChannelNotFound => write!(f, "Channel Not Found"),

            CustomError::ChannelNotPresence => write!(f, "This Channel Not Presence Channel"),
            CustomError::ChannelsNotFound => write!(f, "Channels is Empty"),

            CustomError::EventChannelEmpty => write!(f, "Event Channel or Channels Field Cannot Be Empty"),
            CustomError::NotFound => write!(f, "Pusher App Not Found"),
            CustomError::AppKeyNotFound => write!(f, "There is no app with the app_key you specified"),

            CustomError::AppIdNotFound => write!(f, "There is no app with the app_id you specified"),
            CustomError::AuthKeyMismatch => write!(f, "Auth credentials is wrong"),
            CustomError::AuthSignatureError => write!(f, "Invalid Auth Signature."),
        }
    }
}
