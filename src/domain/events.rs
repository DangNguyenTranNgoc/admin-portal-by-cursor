use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum UserEvent {
    UserCreated(UserEventPayload),
    UserUpdated(UserEventPayload),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserEventPayload {
    pub user_id: i64,
    pub email: String,
    pub occurred_at: DateTime<Utc>,
    #[serde(default)]
    pub metadata: serde_json::Value,
}
