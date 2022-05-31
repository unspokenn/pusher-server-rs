use crate::app::{Channel, PusherQuery, HashMap, Serialize};

#[derive(Serialize, Clone)]
pub(crate) struct ChannelsResponse {
    pub channels: HashMap<String, Info>,
}

impl From<(HashMap<String, Channel>, &PusherQuery)> for ChannelsResponse {
    fn from((c, q): (HashMap<String, Channel>, &PusherQuery)) -> Self {
        let result = if let Some(ref prefix) = q.filter_by_prefix {
            c.iter().filter(|(channel_name, channel)| !channel.is_empty() && channel_name.starts_with(prefix.to_owned().as_str()))
                .map(move |(channel_name, channel)| (channel_name.to_owned(), Info::from((channel, q)))).collect::<HashMap<String, Info>>()
        } else {
            c.iter().filter(|(_channel_name, channel)| !channel.is_empty())
                .map(|(channel_name, channel)| (channel_name.to_owned(), Info::from((channel, q)))).collect::<HashMap<String, Info>>()
        };

        Self {
            channels: result
        }
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct ChannelResponse {
    pub occupied: bool,
    #[serde(flatten)]
    pub info: Info,
}

impl From<(&Channel, &PusherQuery)> for ChannelResponse {
    fn from((c, q): (&Channel, &PusherQuery)) -> Self {
        Self {
            occupied: c.subscriptions_count() > 0,
            info: Info::from((c, q)),
        }
    }
}

#[derive(Serialize, Clone)]
pub(crate) struct Info {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_count: Option<usize>,
    pub subscription_count: usize,
}

impl From<(&Channel, &PusherQuery)> for Info {
    fn from((c, q): (&Channel, &PusherQuery)) -> Self {
        Self {
            user_count: if q.is_channel_presence() { c.users_count() } else { None },
            subscription_count: c.subscriptions_count(),
        }
    }
}
