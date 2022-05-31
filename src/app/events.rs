use crate::app::{as_json_string, ConnectionInfo, Pusher, HashSet, PresenceInformation, PresenceUser, RemovedMember, Deserialize, Serialize, CustomError};

#[repr(C)]
#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct EventRequestBody {
    pub name: String,
    pub data: String,
    pub channels: Option<HashSet<String>>,
    pub channel: Option<String>,
    pub socket_id: Option<String>,
}

impl EventRequestBody {
    #[inline(always)]
    pub(crate) async fn payload_publish(&self, pusher: Pusher) -> Result<(), warp::Rejection> {
        match &self.channel {
            None => {
                match &self.channels {
                    None => {
                        return Err(warp::reject::custom(CustomError::EventChannelEmpty));
                    }
                    Some(channels) => {
                        for channel in channels {
                            self.publish(pusher.clone(), channel.to_owned()).await;
                        }
                    }
                }
            }
            Some(channel) => {
                self.publish(pusher, channel.to_owned()).await;
            }
        }
        Ok(())
    }
    #[inline(always)]
    async fn publish(&self, pusher: Pusher, channel_name: String) {
        let event = ServerEvent::ChannelEvent(CustomEvent {
            event: self.name.to_owned(),
            data: self.data.to_owned().into(),
            channel: channel_name.to_owned(),
            user_id: self.socket_id.to_owned(),
        });
        if let Some(channel) = pusher.channels.read().await.get(&channel_name) {
            channel.publish(event).await.unwrap();
        }
    }
}

#[repr(C)]
#[derive(Debug, Deserialize)]
#[serde(from = "ClientEventJSON")]
#[allow(dead_code)]
pub(crate) enum ClientEvent {
    Subscribe {
        channel: String,
        auth: Option<String>,
        channel_data: Option<serde_json::Value>,
    },
    Unsubscribe {
        channel: String,
    },
    Ping,
    ChannelEvent {
        event: String,
        channel: String,
        data: serde_json::Value,
    },
}

#[repr(C)]
#[derive(Debug, Deserialize)]
#[serde(tag = "event", content = "data")]
enum PusherClientEventJSON {
    #[serde(rename = "pusher:subscribe")]
    Subscribe {
        channel: String,
        auth: Option<String>,
        channel_data: Option<serde_json::Value>,
    },

    #[serde(rename = "pusher:unsubscribe")]
    Unsubscribe { channel: String },

    #[serde(rename = "pusher:ping")]
    Ping(Option<serde_json::Value>),
}

impl From<PusherClientEventJSON> for PusherClientEvent {
    fn from(json: PusherClientEventJSON) -> Self {
        use PusherClientEventJSON::*;
        match json {
            Subscribe {
                channel,
                auth,
                channel_data,
            } => PusherClientEvent::Subscribe {
                channel,
                auth,
                channel_data,
            },
            Unsubscribe { channel } => PusherClientEvent::Unsubscribe { channel },
            Ping(_) => PusherClientEvent::Ping,
        }
    }
}

#[repr(C)]
#[derive(Debug, Deserialize)]
#[serde(from = "PusherClientEventJSON")]
enum PusherClientEvent {
    Subscribe {
        channel: String,
        auth: Option<String>,
        channel_data: Option<serde_json::Value>,
    },
    Unsubscribe {
        channel: String,
    },
    Ping,
}

#[repr(C)]
#[derive(Debug, Deserialize)]
struct CustomClientEvent {
    event: String,
    channel: String,
    data: serde_json::Value,
}

#[repr(C)]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ClientEventJSON {
    PusherEvent(PusherClientEvent),
    CustomEvent(CustomClientEvent),
}

impl From<ClientEventJSON> for ClientEvent {
    fn from(json: ClientEventJSON) -> Self {
        use ClientEventJSON::*;
        use PusherClientEvent::*;
        match json {
            PusherEvent(Subscribe {
                channel,
                auth,
                channel_data,
            }) => ClientEvent::Subscribe {
                channel,
                auth,
                channel_data,
            },
            PusherEvent(Unsubscribe { channel }) => ClientEvent::Unsubscribe { channel },
            PusherEvent(Ping) => ClientEvent::Ping,
            CustomEvent(CustomClientEvent { event, channel, data }) => {
                ClientEvent::ChannelEvent { event, channel, data }
            }
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct CustomEvent {
    pub event: String,
    pub channel: String,
    #[serde(with = "as_json_string")]
    pub data: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "event")]
#[allow(dead_code)]
pub(crate) enum ServerEvent {
    #[serde(rename = "pusher:connection_established")]
    ConnectionEstablished {
        #[serde(with = "as_json_string")]
        data: ConnectionInfo,
    },

    #[serde(rename = "pusher:error")]
    Error {
        message: String,
        code: Option<u16>,
    },

    #[serde(rename = "pusher:pong")]
    Pong,

    #[serde(rename = "pusher_internal:subscription_succeeded")]
    SubscriptionSucceeded {
        channel: String,
        #[serde(with = "as_json_string")]
        data: Option<PresenceInformation>,
    },

    #[serde(rename = "pusher_internal:member_added")]
    MemberAdded {
        channel: String,
        #[serde(with = "as_json_string")]
        data: PresenceUser,
    },

    #[serde(rename = "pusher_internal:member_removed")]
    MemberRemoved {
        channel: String,
        #[serde(with = "as_json_string")]
        data: RemovedMember,
    },

    ChannelEvent(CustomEvent),
}

impl From<ServerEvent> for warp::ws::Message {
    fn from(event: ServerEvent) -> warp::ws::Message {
        warp::ws::Message::text(serde_json::to_string(&event).unwrap())
    }
}
