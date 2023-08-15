// domain for messaging feature
use chrono::{DateTime, Utc};

pub struct Message {
    pub id: i64,
    pub post_id: i64,
    pub thread_id: Option<i64>,
    pub user_id: String,
    pub content: String,
    pub create_dt: DateTime<Utc>,
    pub update_dt: DateTime<Utc>,
}
