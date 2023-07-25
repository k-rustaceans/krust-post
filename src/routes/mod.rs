mod home;

use axum::{
    Router,
    body::Body,
    routing::get
};
use crate::routes::home::index;

pub fn create_routes() -> Router<(), Body> {
    Router::new().route("/", get(index))
}
