use axum::Json;
use crate::domain::commands::CreatePost;

pub async fn create_post(body: Json<CreatePost>) -> Json<CreatePost> {
    body
}
