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

	use std::{thread::sleep, time::Duration};

	use async_nats::jetstream::{
		self,
		consumer::{AckPolicy, DeliverPolicy, PushConsumer, ReplayPolicy},
	};
	use futures_util::StreamExt;
	use serde::{Deserialize, Serialize};

	#[derive(Serialize, Deserialize)]
	struct TestData {
		data: String,
	}

	#[tokio::test]
	async fn test_round_trip() {
		let client = async_nats::connect("nats://localhost:4222").await.unwrap();

		// Test Out Multiple consumption of messages
		let mut subscriber = client.subscribe("messages".into()).await.unwrap();
		let mut subscriber2 = client.subscribe("messages".into()).await.unwrap();

		let test_data = "dsadsadsdas".to_string();
		client.publish("messages".into(), test_data.clone().into()).await.unwrap();

		let (f, s) = tokio::join!(
			tokio::spawn({
				let test_data = test_data.clone();
				async move {
					if let Some(message) = subscriber.next().await {
						println!("Run First subscriber");
						let payload = String::from_utf8(message.payload.to_vec()).unwrap();
						assert_eq!(test_data, payload)
					}
				}
			}),
			tokio::spawn({
				let test_data = test_data.clone();
				async move {
					if let Some(message) = subscriber2.next().await {
						println!("Run Second subscriber");
						let payload = String::from_utf8(message.payload.to_vec()).unwrap();
						assert_eq!(test_data, payload)
					}
				}
			})
		);
		f.unwrap();
		s.unwrap();
	}

	#[tokio::test]
	async fn test_round_trip_jetstream() {
		let client = async_nats::connect("nats://0.0.0.0:4222").await.unwrap();

		let inbox = client.new_inbox();

		// println!("{}", inbox);
		// Access the JetStream Context for managing streams and consumers as well as for publishing and subscription convenience methods.
		let jetstream = jetstream::new(client);
		let stream_name = String::from("EVENTS");

		let stream = jetstream
			.create_stream(jetstream::stream::Config {
				name: stream_name,
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

		let consumer = stream
			.create_consumer(jetstream::consumer::push::Config {
				deliver_subject: inbox,
				// inactive_threshold: Duration::from_secs(60),
				deliver_policy: DeliverPolicy::All,
				ack_policy: AckPolicy::Explicit,

				durable_name: Some("MyGroup".to_string()),
				deliver_group: Some("youd".to_string()),
				..Default::default()
			})
			.await
			.unwrap();

		let mut messages = consumer.messages().await.unwrap().take(10);
		while let Some(message) = messages.next().await {
			let message = message.unwrap();
			println!(
				"got message on subject {} with payload {:?}",
				message.subject,
				std::str::from_utf8(&message.payload).unwrap()
			);

			// acknowledge the message
			message.ack().await.unwrap();
		}
	}
}
