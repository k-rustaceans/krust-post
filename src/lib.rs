pub mod domain;
mod routes;
mod common;

use crate::routes::create_routes;

pub async fn run() {
    let app = create_routes();

    // hardcoded: localhost IP and 3000 port
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
