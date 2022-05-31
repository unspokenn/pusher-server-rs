pub(crate) mod as_json_string {
    use serde::{Serialize, Serializer};
    use serde_json;

    pub(crate) fn serialize<T: Serialize, S: Serializer>(value: &T, serializer: S) -> Result<S::Ok, S::Error> {
        let json = serde_json::to_string(value).map_err(serde::ser::Error::custom)?;
        json.serialize(serializer)
    }
}

pub(crate) mod arc_rwlock_serde {
    use std::ops::Deref;
    use serde::{Serialize, Serializer};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    pub(crate) fn serialize<S, T>(val: &Arc<RwLock<T>>, s: S) -> Result<S::Ok, S::Error>
    where S: Serializer,
          T: Serialize,
    {
        T::serialize(&*val.blocking_read().deref(), s)
    }
}
