use std::env;
use std::ops::{Deref, DerefMut};
use std::sync::OnceLock;
use std::{mem, sync::Arc};

use event_driven_library::responses::BaseError;

use pulsar::message::proto::command_subscribe::SubType;
use pulsar::{message::proto, producer, Pulsar};
use pulsar::{Consumer, DeserializeMessage, Producer, TokioExecutor};
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

pub struct QueuePubExecutor {
	pub(crate) producer: Producer<TokioExecutor>,
}

impl QueuePubExecutor {
	pub async fn new(
		topic: &str,
		producer_name: &str,
	) -> Self {
		Self {
			producer: {
				dotenv::dotenv().ok();
				let url = &env::var("QUEUE_URL").expect("QUEUE_URL must be set");
				let pulsar: Pulsar<_> = Pulsar::builder(url, TokioExecutor).build().await.unwrap();

				pulsar
					.producer()
					.with_topic(topic)
					.with_name(producer_name)
					.with_options(producer::ProducerOptions {
						schema: Some(proto::Schema {
							r#type: proto::schema::Type::String as i32,
							..Default::default()
						}),
						..Default::default()
					})
					.build()
					.await
					.unwrap()
			},
		}
	}
}

impl From<QueuePubExecutor> for Arc<RwLock<QueuePubExecutor>> {
	fn from(value: QueuePubExecutor) -> Self {
		Arc::new(RwLock::new(value))
	}
}

impl Deref for QueuePubExecutor {
	type Target = Producer<TokioExecutor>;
	fn deref(&self) -> &Self::Target {
		&self.producer
	}
}
impl DerefMut for QueuePubExecutor {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.producer
	}
}

pub struct QueueConExecutor<T: DeserializeMessage> {
	pub(crate) consumer: Consumer<T, TokioExecutor>,
}

impl<T: DeserializeMessage + 'static> QueueConExecutor<T> {
	pub async fn new(
		topic: &str,
		subscription: &str,
	) -> Self {
		dotenv::dotenv().ok();
		let url = &env::var("QUEUE_URL").expect("QUEUE_URL must be set");
		let pulsar: Pulsar<_> = Pulsar::builder(url, TokioExecutor).build().await.unwrap();
		Self {
			consumer: pulsar
				.consumer()
				.with_topic(topic)
				.with_consumer_name("test_consumer")
				.with_subscription_type(SubType::Shared)
				.with_subscription(subscription)
				.build()
				.await
				.unwrap(),
		}
	}
}

impl<T: DeserializeMessage + 'static> Deref for QueueConExecutor<T> {
	type Target = Consumer<T, TokioExecutor>;
	fn deref(&self) -> &Self::Target {
		&self.consumer
	}
}
impl<T: DeserializeMessage + 'static> DerefMut for QueueConExecutor<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.consumer
	}
}

#[cfg(test)]
mod test {

	use futures::TryStreamExt;
	use pulsar::{producer, Error as PulsarError, SerializeMessage};
	use pulsar::{DeserializeMessage, Payload};
	use serde::{Deserialize, Serialize};

	use super::QueuePubExecutor;

	use super::QueueConExecutor;

	#[derive(Serialize, Deserialize)]
	struct TestData {
		data: String,
	}
	impl SerializeMessage for TestData {
		fn serialize_message(input: Self) -> Result<producer::Message, PulsarError> {
			let payload =
				serde_json::to_vec(&input).map_err(|e| PulsarError::Custom(e.to_string()))?;
			Ok(producer::Message {
				payload,
				..Default::default()
			})
		}
	}

	async fn set_up<T: DeserializeMessage + 'static>(
		topic: &str,
		subscription: &str,
	) -> Result<(), PulsarError> {
		use pulsar::proto::MessageIdData;
		let mut consumer: QueueConExecutor<T> = QueueConExecutor::new(topic, subscription).await;

		let vec_message_ids: Vec<MessageIdData> = consumer.get_last_message_id().await.unwrap();
		let last = vec_message_ids.last().unwrap().clone();
		consumer.ack_with_id(topic, last).await?;

		Ok(())
	}

	#[tokio::test]
	async fn test_publish() {
		let topic = "test";

		let mut producer = QueuePubExecutor::new(topic, "test_producer1").await;

		match producer
			.send(TestData {
				data: "data".to_string(),
			})
			.await
			.unwrap()
			.await
		{
			Ok(val) => {
				println!("{:?}", val);
			}
			Err(err) => panic!("{}", err),
		}
	}

	impl DeserializeMessage for TestData {
		type Output = Result<TestData, serde_json::Error>;

		fn deserialize_message(payload: &Payload) -> Self::Output {
			serde_json::from_slice(&payload.data)
		}
	}

	#[tokio::test]
	//TODO get this test passed!
	async fn test_round_trip() {
		'_given: {
			let topic = "test";
			let subscription = "test_subscription";
			set_up::<TestData>(topic, subscription).await.unwrap();
			'_when: {
				let mut producer = QueuePubExecutor::new(topic, "test_producer2").await;
				producer
					.send(TestData {
						data: "TestDataForRoundTrip23".to_string(),
					})
					.await
					.unwrap();

				let mut consumer: QueueConExecutor<TestData> =
					QueueConExecutor::new(topic, subscription).await;

				if let Some(msg) = consumer.try_next().await.unwrap() {
					let data = msg.deserialize().unwrap();
					consumer.ack(&msg).await.unwrap();
					assert_eq!(data.data, "TestDataForRoundTrip23".to_string());
				} else {
					panic!("Test Failed!")
				}
			}
		}
	}
}
