use std::sync::OnceLock;

use async_nats::Client;
use event_driven_library::responses::BaseError;
use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::config::Config;

pub struct Dependency;
impl Dependency {
	pub fn config(&self) -> &'static Config {
		config()
	}
}

pub fn config() -> &'static Config {
	static CONFIG: OnceLock<Config> = OnceLock::new();
	let config = match CONFIG.get() {
		None => {
			let config = Config::new().unwrap();

			CONFIG.get_or_init(|| config)
		}
		Some(config) => config,
	};
	config
}

pub fn dependency() -> &'static Dependency {
	static DEPENDENCY: OnceLock<Dependency> = OnceLock::new();
	let dp = match DEPENDENCY.get() {
		None => {
			let dependency = Dependency;

			DEPENDENCY.get_or_init(|| dependency)
		}
		Some(dependency) => dependency,
	};
	dp
}

pub async fn connection_pool() -> &'static PgPool {
	static POOL: OnceLock<PgPool> = OnceLock::new();

	let p = match POOL.get() {
		None => {
			let url: &String = &config().database_url;
			let pool = PgPoolOptions::new()
				.max_connections(30)
				.connect(url)
				.await
				.map_err(|err| BaseError::DatabaseConnectionError(Box::new(err)))
				.unwrap();
			POOL.get_or_init(|| pool)
		}
		Some(pool) => pool,
	};
	p
}

pub async fn queue_client() -> &'static Client {
	static CLIENT: OnceLock<Client> = OnceLock::new();

	let c = match CLIENT.get() {
		None => {
			let cl = async_nats::ConnectOptions::new().name("r").connect(&config().queue_url).await.unwrap();
			CLIENT.get_or_init(|| cl)
		}
		Some(c) => c,
	};
	c
}
