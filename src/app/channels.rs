use futures::future;
use tokio::sync::mpsc;
use crate::app::{ServerEvent, HashMap, Serialize};

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct Subscription {
    #[serde(skip)]
    pub sender: mpsc::Sender<ServerEvent>,
    pub data: Option<serde_json::Value>,
    pub user_id: Option<String>,
}

impl Subscription {
    pub async fn publish(&self, msg: ServerEvent) -> Result<(), String> {
        self.sender.send(msg).await.map_err(|err| format!("{}", err))
    }
}

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
pub(crate) enum Channel {
    Public {
        subscriptions: HashMap<String, Subscription>,
    },
    Private {
        subscriptions: HashMap<String, Subscription>,
    },
    Presence {
        subscriptions: HashMap<String, Subscription>,
        users: HashMap<String, serde_json::Value>,
    },
}

impl Channel {
    pub(crate) async fn publish(&self, event: ServerEvent) -> Result<(), String> {
        let messages = self.subscriptions().values().map(|sub| sub.publish(event.clone()));
        future::join_all(messages).await;
        Ok(())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.subscriptions().is_empty()
    }

    pub(crate) fn subscriptions_count(&self) -> usize {
        self.subscriptions().len()
    }

    pub(crate) fn add_subscription(&mut self, socket_id: &str, subscription: Subscription) {
        self.subscriptions_mut().insert(socket_id.to_owned(), subscription);
    }

    pub(crate) fn remove_subscription(&mut self, socket_id: &str) {
        self.subscriptions_mut().remove(socket_id);
    }

    pub(crate) fn users_count(&self) -> Option<usize> {
        match self {
            Channel::Public { .. } => None,
            Channel::Private { .. } => None,
            Channel::Presence { users, .. } => Some(users.len()),
        }
    }

    pub(crate) fn subscriptions(&self) -> &HashMap<String, Subscription> {
        match self {
            Channel::Public { subscriptions } => subscriptions,
            Channel::Private { subscriptions } => subscriptions,
            Channel::Presence { subscriptions, .. } => subscriptions,
        }
    }

    pub(crate) fn subscriptions_mut(&mut self) -> &mut HashMap<String, Subscription> {
        match self {
            Channel::Public { subscriptions } => subscriptions,
            Channel::Private { subscriptions } => subscriptions,
            Channel::Presence { subscriptions, .. } => subscriptions,
        }
    }
}

impl From<String> for Channel {
    fn from(name: String) -> Channel {
        match name.as_str().splitn(2, "-").collect::<Vec<&str>>().as_slice() {
            ["private", ..] => Channel::Private {
                subscriptions: HashMap::default(),
            },
            ["presence", ..] => Channel::Presence {
                subscriptions: HashMap::default(),
                users: HashMap::default(),
            },
            _ => Channel::Public {
                subscriptions: HashMap::default(),
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct PresenceInformation {
    ids: Vec<String>,
    hash: HashMap<String, HashMap<String, String>>,
    count: u32,
}

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct PresenceUser {
    #[serde(rename = "user_id")]
    id: String,
    #[serde(rename = "user_info")]
    info: serde_json::Value,
}

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
pub(crate) struct RemovedMember {
    #[serde(rename = "user_id")]
    id: String,
}
