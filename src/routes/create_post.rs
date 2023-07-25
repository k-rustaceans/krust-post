use axum::Json;
use crate::domain::commands::CreatePost;

pub async fn create_post(Json(post): Json<CreatePost>) -> Json<CreatePost> {
    Json(post)
}
