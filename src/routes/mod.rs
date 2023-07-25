mod home;
mod create_post;

use axum::{Router, body::Body, routing::{
    get,
    post,
}, Extension, middleware};
use axum::http::Method;
use tower_http::cors::{Any, CorsLayer};
use crate::common::middleware_custom_header::{read_middleware_custom_header, set_middleware_custom_header};
use crate::common::shared_data::SharedData;
use crate::routes::create_post::create_post;
use crate::routes::home::index;

pub fn create_routes() -> Router<(), Body> {
    // used tower-http middleware for cors
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    // shared middleware data(authentication, etc..)
    let shared_data = SharedData {
        message: "".to_owned()
    };

    Router::new()
        .route("/custom_header", get(read_middleware_custom_header))
        .route_layer(middleware::from_fn(set_middleware_custom_header))
        .route("/", get(index))
        .route("/post", post(create_post))
        .layer(cors)
        .layer(Extension(shared_data))
}
