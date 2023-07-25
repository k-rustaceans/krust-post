mod home;
mod create_post;

use axum::{
    Router,
    body::Body,
    routing::{
        get,
        post
    }
};
use crate::routes::home::index;

pub fn create_routes() -> Router<(), Body> {
    Router::new()
        .route("/", get(index))
        .route("/post", post(create_post))
}
