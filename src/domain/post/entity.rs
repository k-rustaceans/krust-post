use chrono::{DateTime, Utc};
use event_driven_library::Entity;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, Serialize, Deserialize,Entity!)]
pub struct Post {
    pub id: i64,
    pub user_id: String,
    pub create_dt: DateTime<Utc>,
    pub update_dt: DateTime<Utc>,
    pub state: PostState,
}
#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, Serialize, Deserialize)]
pub enum PostState {
    #[default]
    Created,
    Deleted,
}

#[derive(Clone, PartialEq, Eq, Debug, Default, Hash, Serialize, Deserialize,Entity!)]
pub struct PostContent {
    pub id: i64,
    pub post_id: i64,
    pub title: String,
    pub content: String,
    pub create_dt: DateTime<Utc>,
}
pub struct PostLike {
    pub user_id: String,
    pub post_id: i64,
}
