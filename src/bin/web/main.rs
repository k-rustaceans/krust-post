pub mod routers;

use std::{env, net::SocketAddr, str::FromStr};

use axum::{
	http::{HeaderValue, Method},
	Router,
};

use post::{
	dependencies::{config, queue_client},
	domain::thread::{ThreadState, ThreadStateWrapper},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
	println!("Environment Variable Is Being Set...");
	dotenv::dotenv().ok();

	// ! Tracing
	tracing_subscriber::registry()
		.with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
			// axum logs rejections from built-in extractors with the `axum::rejection`
			// target, at `TRACE` level. `axum::rejection=trace` enables showing those events
			"tracing=debug,tower_http=debug,axum::rejection=trace".into()
		}))
		.with(tracing_subscriber::fmt::layer())
		.init();

	// ! Connection
	println!("Connections Are Being Pooled...");

	// let bus = Boostrap::message_bus().await;
	let chat_state: ThreadStateWrapper = ThreadState {
		room: Default::default(),
		queue_client: queue_client().await.into(),
	}
	.into();

	let routers = Router::new()
		// .layer(middleware::from_fn(middlewares::auth))
		.nest("/posts", routers::post())
		.with_state(chat_state);

	let service_name = "/krust-post";
	let app = Router::new()
		.nest_service(service_name, routers)
		.layer(
			CorsLayer::new()
				.allow_origin(config().allow_origins.parse::<HeaderValue>().unwrap())
				.allow_methods([Method::GET, Method::POST, Method::PATCH, Method::PUT, Method::DELETE]),
		)
		.layer(TraceLayer::new_for_http());

	println!("Start Web Server...");
	axum::Server::bind(&SocketAddr::from_str(&env::var("SERVER_IP_PORT").unwrap_or("0.0.0.0:80".into())).unwrap())
		.serve(app.into_make_service())
		.await
		.unwrap();
}
