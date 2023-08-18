use std::env;
use std::sync::OnceLock;
use std::{mem, sync::Arc};

use event_driven_library::responses::BaseError;
use sqlx::postgres::PgPoolOptions;
use sqlx::{postgres::PgPool, Postgres, Transaction};

use tokio::sync::RwLock;

pub struct DatabaseExecutor {
	pool: &'static PgPool,
	transaction: Option<Transaction<'static, Postgres>>,
}

impl DatabaseExecutor {
	pub async fn new() -> Self {
		Self {
			pool: connection_pool().await,
			transaction: None,
		}
	}
	pub fn transaction(&mut self) -> &mut Transaction<'static, Postgres> {
		match self.transaction.as_mut() {
			Some(trx) => trx,
			None => panic!("Transaction Has Not Begun!"),
		}
	}
	pub fn connection(&self) -> &PgPool {
		self.pool
	}

	pub(crate) async fn begin(&mut self) -> Result<(), BaseError> {
		match self.transaction.as_mut() {
			None => {
				self.transaction = Some(
					self.pool
						.begin()
						.await
						.map_err(|err| BaseError::DatabaseConnectionError(Box::new(err)))?,
				);
				Ok(())
			}
			Some(_trx) => {
				println!("Transaction Begun Already!");
				Err(BaseError::TransactionError)?
			}
		}
	}

	pub(crate) async fn commit(&mut self) -> Result<(), BaseError> {
		if self.transaction.is_none() {
			panic!("Tranasction Has Not Begun!");
		};

		let trx = mem::take(&mut self.transaction).unwrap();
		trx.commit().await.map_err(|err| {
			eprintln!("Error occurred during commit operation : {:?}", err);
			BaseError::DatabaseConnectionError(Box::new(err))
		})
	}
	pub(crate) async fn rollback(&mut self) -> Result<(), BaseError> {
		if self.transaction.is_none() {
			panic!("Tranasction Has Not Begun!");
		};

		let trx = mem::take(&mut self.transaction).unwrap();
		trx.rollback()
			.await
			.map_err(|err| BaseError::DatabaseConnectionError(Box::new(err)))
	}
}

impl From<DatabaseExecutor> for Arc<RwLock<DatabaseExecutor>> {
	fn from(value: DatabaseExecutor) -> Self {
		Arc::new(RwLock::new(value))
	}
}

pub async fn connection_pool() -> &'static PgPool {
	static POOL: OnceLock<PgPool> = OnceLock::new();
	dotenv::dotenv().ok();
	let url = &env::var("DATABASE_URL").expect("DATABASE_URL must be set");
	let p = match POOL.get() {
		None => {
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
