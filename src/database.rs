use async_nats::{Client, Subscriber};
use bytes::Bytes;
use event_driven_library::responses::BaseError;

use std::{mem, ops::Deref, sync::Arc};

use sqlx::{postgres::PgPool, Postgres, Transaction};

use crate::{
	dependencies::{connection_pool, queue_client},
	domain::thread::schemas::ClientMessage,
};

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
				self.transaction = Some(self.pool.begin().await.map_err(|err| BaseError::DatabaseConnectionError(Box::new(err)))?);
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
		trx.rollback().await.map_err(|err| BaseError::DatabaseConnectionError(Box::new(err)))
	}
}

impl From<DatabaseExecutor> for Arc<RwLock<DatabaseExecutor>> {
	fn from(value: DatabaseExecutor) -> Self {
		Arc::new(RwLock::new(value))
	}
}

pub struct QueueClient(&'static Client);

impl QueueClient {
	pub async fn send(
		&self,
		msg: ClientMessage,
	) -> std::result::Result<(), async_nats::PublishError> {
		let msg: Bytes = serde_json::to_string(&msg).unwrap().into();

		self.publish("chat".into(), msg).await
	}
}

impl From<&'static Client> for QueueClient {
	fn from(value: &'static Client) -> Self {
		QueueClient(value)
	}
}

impl From<QueueClient> for Arc<RwLock<QueueClient>> {
	fn from(value: QueueClient) -> Self {
		Arc::new(RwLock::new(value))
	}
}

impl Deref for QueueClient {
	type Target = &'static Client;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub struct QueueConExecutor(Subscriber);

impl QueueConExecutor {
	pub async fn new(topic: &str) -> Self {
		let client: &'static Client = queue_client().await;
		Self(client.subscribe(topic.into()).await.unwrap())
	}
}

#[cfg(test)]
pub mod test {
	use async_nats::jetstream::{self, consumer::PullConsumer};
	use futures::StreamExt;
	use futures::TryStreamExt;
	use serde::{Deserialize, Serialize};

	#[derive(Serialize, Deserialize)]
	struct TestData {
		data: String,
	}

	#[tokio::test]
	async fn test_round_trip_jetstream() {
		let client = async_nats::connect("nats://0.0.0.0:4222").await.unwrap();

		// println!("{}", inbox);
		// Access the JetStream Context for managing streams and consumers as well as for publishing and subscription convenience methods.
		let jetstream = jetstream::new(client);
		let stream_name = String::from("EVENTS");

		let stream = jetstream
			.get_or_create_stream(jetstream::stream::Config {
				name: stream_name,
				max_messages: 10_000,
				subjects: vec!["events.>".to_string()],

				..Default::default()
			})
			.await
			.unwrap();

		// Publish a few messages for the example.
		for i in 0..10 {
			jetstream
				.publish(format!("events.{i}"), "data".into())
				// The first `await` sends the publish
				.await
				.unwrap()
				// The second `await` awaits a publish acknowledgement.
				// This can be skipped (for the cost of processing guarantee)
				// or deferred to not block another `publish`
				.await
				.unwrap();
		}

		// DeliverPolicy::ByStartTime { start_time: () }

		let consumer: PullConsumer = stream
			.get_or_create_consumer(
				"consumer",
				jetstream::consumer::pull::Config {
					// inactive_threshold: Duration::from_secs(60),
					durable_name: Some("MyGroup".to_string()),

					..Default::default()
				},
			)
			.await
			.unwrap();

		// let mut messages = consumer.messages().await.unwrap().take(10);
		// let a = consumer.messages().await.unwrap().next().await;

		let mut messages = consumer.messages().await.unwrap().take(10);
		while let Ok(Some(message)) = messages.try_next().await {
			println!(
				"got message on subject {} with payload {:?}",
				message.subject,
				std::str::from_utf8(&message.payload).unwrap()
			);

			// acknowledge the message
			message.ack().await.unwrap();
		}
	}

	#[tokio::test]
	async fn test_round_trip_jet_stream_batch() {
		let client = async_nats::connect("nats://0.0.0.0:4222").await.unwrap();

		// println!("{}", inbox);
		// Access the JetStream Context for managing streams and consumers as well as for publishing and subscription convenience methods.
		let jetstream = jetstream::new(client);
		let stream_name = String::from("TEST2");

		let stream = jetstream
			.get_or_create_stream(jetstream::stream::Config {
				name: stream_name,
				max_messages: 10_000,
				subjects: vec!["events2.>".to_string()],

				..Default::default()
			})
			.await
			.unwrap();

		// Publish a few messages for the example.
		for i in 0..100 {
			jetstream
				.publish(format!("events2.{i}"), "data".into())
				// The first `await` sends the publish
				.await
				.unwrap()
				// The second `await` awaits a publish acknowledgement.
				// This can be skipped (for the cost of processing guarantee)
				// or deferred to not block another `publish`
				.await
				.unwrap();
		}

		// DeliverPolicy::ByStartTime { start_time: () }

		let consumer: PullConsumer = stream
			.get_or_create_consumer(
				"consumer",
				jetstream::consumer::pull::Config {
					// inactive_threshold: Duration::from_secs(60),
					durable_name: Some("MyGroup".to_string()),

					..Default::default()
				},
			)
			.await
			.unwrap();

		// let mut messages = consumer.messages().await.unwrap().take(10);
		// let a = consumer.messages().await.unwrap().next().await;

		let mut cnt = 0;
		let mut batches = consumer.sequence(10).unwrap().take(10);
		while let Ok(Some(mut batch)) = batches.try_next().await {
			println!("iteration: {}", cnt + 1);
			if cnt == 10 {
				break;
			}
			while let Ok(Some(message)) = batch.try_next().await {
				println!(
					"got message on subject {} with payload {:?}",
					message.subject,
					std::str::from_utf8(&message.payload).unwrap()
				);

				// acknowledge the message
				message.ack().await.unwrap();
			}
			cnt += 1;
		}
	}

	#[tokio::test]
	async fn test_round_trip_jet_stream_multi_consumer() {
		let client = async_nats::connect("nats://0.0.0.0:4222").await.unwrap();

		// println!("{}", inbox);
		// Access the JetStream Context for managing streams and consumers as well as for publishing and subscription convenience methods.
		let jetstream = jetstream::new(client);
		let stream_name = String::from("TEST2");

		let stream = jetstream
			.get_or_create_stream(jetstream::stream::Config {
				name: stream_name,
				max_messages: 10_000,
				subjects: vec!["events2.>".to_string()],

				..Default::default()
			})
			.await
			.unwrap();

		// Publish a few messages for the example.
		for i in 0..10 {
			jetstream
				.publish(format!("events2.{i}"), "data".into())
				// The first `await` sends the publish
				.await
				.unwrap()
				// The second `await` awaits a publish acknowledgement.
				// This can be skipped (for the cost of processing guarantee)
				// or deferred to not block another `publish`
				.await
				.unwrap();
		}

		tokio::join!(
			{
				let stream = stream.clone();
				async move {
					let consumer: PullConsumer = stream
						.get_or_create_consumer(
							"consumer1",
							jetstream::consumer::pull::Config {
								// inactive_threshold: Duration::from_secs(60),
								durable_name: Some("test_group1".to_string()),

								..Default::default()
							},
						)
						.await
						.unwrap();
					let mut cnt = 0;
					let mut messages = consumer.messages().await.unwrap().take(10);

					while let Ok(Some(message)) = messages.try_next().await {
						println!("iteration: from consumer1 {}", cnt + 1);
						if cnt == 10 {
							break;
						}
						println!(
							"got message on subject {} with payload {:?}",
							message.subject,
							std::str::from_utf8(&message.payload).unwrap()
						);

						// acknowledge the message
						message.ack().await.unwrap();
						cnt += 1;
					}
				}
			},
			{
				let stream = stream.clone();
				async move {
					let consumer: PullConsumer = stream
						.get_or_create_consumer(
							"consumer2",
							jetstream::consumer::pull::Config {
								// inactive_threshold: Duration::from_secs(60),
								durable_name: Some("test_group2".to_string()),

								..Default::default()
							},
						)
						.await
						.unwrap();
					let mut cnt = 0;
					let mut messages = consumer.messages().await.unwrap().take(10);

					while let Ok(Some(message)) = messages.try_next().await {
						println!("iteration: from consumer2 {}", cnt + 1);
						if cnt == 10 {
							break;
						}
						println!(
							"got message on subject {} with payload {:?}",
							message.subject,
							std::str::from_utf8(&message.payload).unwrap()
						);

						// acknowledge the message
						message.ack().await.unwrap();
						cnt += 1;
					}
				}
			}
		);
	}
}
