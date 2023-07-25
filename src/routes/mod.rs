mod home;
mod create_post;

use axum::{
    Router,
    body::Body,
    routing::{
        get,
        post,
    },
};
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use crate::routes::create_post::create_post;
use crate::routes::home::index;

pub fn create_routes() -> Router<(), Body> {
    // used tower-http middleware for cors
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    Router::new()
        .route("/", get(index))
        .route("/post", post(create_post))
        .layer(cors)
}
