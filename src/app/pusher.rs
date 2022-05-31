use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use crate::app::{check_signature, arc_rwlock_serde, HashMap, Deserialize, Serialize, Channel, CustomError};
// use chrono::serde::ts_milliseconds::serialize as to_milli_ts;
use chrono::serde::ts_milliseconds::deserialize as from_milli_ts;

#[repr(C)]
#[derive(Clone, Debug, Serialize, Default)]
pub struct Pusher {
    pub id: u32,
    pub key: String,
    pub secret: String,
    pub name: Option<String>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub capacity: Option<u32>,
    pub client_messages_enabled: Option<bool>,
    pub statistics_enabled: Option<bool>,
    pub allowed_origins: Option<Vec<String>>,
    #[serde(with = "arc_rwlock_serde")]
    pub(crate) channels: Arc<RwLock<HashMap<String, Channel>>>,
}

impl Pusher {
    pub fn new(app_id: u32, app_key: &str, app_secret: &str) -> Self {
        Pusher {
            id: app_id,
            key: app_key.to_owned(),
            secret: app_secret.to_owned(),
            name: None,
            host: None,
            path: None,
            capacity: None,
            client_messages_enabled: None,
            statistics_enabled: None,
            allowed_origins: None,
            channels: Arc::new(RwLock::new(HashMap::default())),
        }
    }
    #[allow(dead_code)]
    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_owned());
    }
    #[allow(dead_code)]
    pub fn set_host(&mut self, host: &str) {
        self.host = Some(host.to_owned());
    }
    #[allow(dead_code)]
    pub fn set_path(&mut self, path: &str) {
        self.path = Some(path.to_owned());
    }
    #[allow(dead_code)]
    pub fn set_capacity(&mut self, capacity: u32) {
        self.capacity = Some(capacity);
    }
    #[allow(dead_code)]
    pub fn set_client_messages_enabled(&mut self, client_messages_enabled: bool) {
        self.client_messages_enabled = Some(client_messages_enabled);
    }
    #[allow(dead_code)]
    pub fn set_statistics_enabled(&mut self, statistics_enabled: bool) {
        self.statistics_enabled = Some(statistics_enabled);
    }
    #[allow(dead_code)]
    pub fn set_allowed_origins(&mut self, allowed_origins: Vec<String>) {
        self.allowed_origins = Some(allowed_origins);
    }
    #[inline(always)]
    fn prepare_auth_body(&self, query: &PusherQuery, path: &str) -> String {
        let format = format!("{path}auth_key={auth_key}&auth_timestamp={timestamp}&auth_version={auth_version}",
            path = path,
            auth_key = self.key,
            timestamp = query.auth_timestamp.timestamp_millis(),
            auth_version = if query.auth_version == 1.0 { "1.0".to_string() } else { query.auth_version.to_string() }
        );

        match &query.body_md5 {
            None => { format }
            Some(body_md5) => { format!("{}&body_md5={}", format, body_md5) }
        }
    }
    #[inline(always)]
    pub(crate) fn ensure_valid_signature(&self, query: &PusherQuery, path: &str) -> Result<Pusher, warp::Rejection> {
        let auth_body = self.prepare_auth_body(query, path);

        check_signature(query.auth_signature.as_str(), self.secret.as_str(), auth_body.as_str()).map(move |_| self.clone())
    }
    #[inline(always)]
    pub(crate) async fn get_channel(&self, name: String) -> Result<Channel, warp::Rejection> {
        if let Some(channel) = self.channels.read().await.get(&name) {
            return Ok(channel.clone());
        }
        Err(warp::reject::custom(CustomError::ChannelNotFound))
    }
    #[inline(always)]
    pub(crate) async fn get_channels(&self) -> Result<HashMap<String, Channel>, warp::Rejection> {
        let channels = self.channels.read().await;
        if channels.len() == 0 {
            return Ok(channels.clone());
        }
        Err(warp::reject::custom(CustomError::ChannelsNotFound))
    }
}

#[repr(C)]
#[derive(Clone, Debug, Serialize)]
pub struct PusherServer {
    apps: HashMap<String, Pusher>,
}

impl PusherServer {
    pub fn new(app: Pusher) -> Self {
        let mut map = HashMap::default();
        map.insert(app.key.to_owned(), app);
        PusherServer {
            apps: map
        }
    }
    #[allow(dead_code)]
    pub fn add(&mut self, app: Pusher) {
        self.apps.insert(app.key.to_owned(), app);
    }
    #[allow(dead_code)]
    pub(crate) fn remove(&mut self, key: &str) {
        self.apps.remove(key);
    }
    #[inline(always)]
    pub(crate) fn find(&self, key: &str) -> Result<Pusher, warp::Rejection> {
        if let Some(pusher) = self.apps.get(key) {
            return Ok(pusher.clone());
        }
        Err(warp::reject::custom(CustomError::NotFound))
    }
    #[inline(always)]
    pub(crate) fn find_by_id(&self, id: u32) -> Result<Pusher, warp::Rejection> {
        for (_, pusher) in self.apps.iter() {
            if pusher.id == id {
                return Ok(pusher.clone());
            }
        }
        Err(warp::reject::custom(CustomError::NotFound))
    }
    #[allow(dead_code)]
    #[inline(always)]
    pub(crate) async fn find_by_id_with_channels(&self, id: u32, channel_name: String) -> Result<(Pusher, Channel), warp::Rejection> {
        for (_, pusher) in self.apps.iter() {
            if pusher.id == id {
                let channel = pusher.get_channel(channel_name).await?;
                return Ok((pusher.clone(), channel));
            }
        }
        Err(warp::reject::custom(CustomError::NotFound))
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ConnectionInfo {
    pub socket_id: String,
    pub activity_timeout: u8,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct PusherQuery {
    pub auth_key: String,
    #[serde(deserialize_with = "from_milli_ts")]
    pub auth_timestamp: DateTime<Utc>,
    pub auth_version: f32,
    pub body_md5: Option<String>,
    pub auth_signature: String,
    pub info: Option<InfoQueryPram>,
    pub filter_by_prefix: Option<String>,
}

impl PusherQuery {
    pub(crate) fn is_channel_presence(&self) -> bool {
        if let Some(info) = &self.info {
            if let Some(filter_by_prefix) = &self.filter_by_prefix {
                if info.user_count && filter_by_prefix.starts_with("presence-") {
                    return true;
                }
            }
        }
        false
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub(crate) struct InfoQueryPram {
    pub user_count: bool,
    pub subscription_count: bool,
}

impl std::str::FromStr for InfoQueryPram {
    type Err = std::string::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let text: Vec<&str> = s.split(',').collect();
        let mut user_count = false;
        let mut subscription_count = false;
        for t in text {
            if !user_count {
                user_count = t == "user_count";
            }
            if !subscription_count {
                subscription_count = t == "subscription_count";
            }
        }
        Ok(InfoQueryPram { user_count, subscription_count })
    }
}
