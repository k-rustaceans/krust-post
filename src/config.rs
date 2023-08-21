use crate::services::response::ServiceError;

pub struct Config {
	/// Which errors we want to log
	pub log_level: String,

	/// Port server is listening to
	pub server_ip_port: String,
	pub queue_url: String,
	pub database_url: String,
	pub allow_origins: String,
}

impl Config {
	pub fn new() -> Result<Config, ServiceError> {
		dotenv::dotenv().ok();
		let queue_url = std::env::var("QUEUE_URL").expect("QUEUE_URL must be set");
		let log_level = std::env::var("LOG_LEVEL").unwrap_or("warn".to_string());
		let server_ip_port = std::env::var("SERVER_IP_PORT").unwrap_or("0.0.0.0:80".into());
		let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
		let allow_origins = std::env::var("ALLOW_ORIGINS").unwrap_or("http://localhost:3000,http://localhost:3001".to_string());

		Ok(Config {
			queue_url,
			log_level,
			server_ip_port,
			database_url,
			allow_origins,
		})
	}
}
