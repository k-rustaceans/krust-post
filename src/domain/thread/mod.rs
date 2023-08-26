pub mod schemas;
use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
	sync::Arc,
};

use async_nats::Subscriber;
// domain for messaging feature
use chrono::{DateTime, Utc};
use tokio::sync::{broadcast, Mutex, MutexGuard};
use uuid::Uuid;

use crate::database::QueueClient;

pub struct MainThreadMessage {
	pub id: Uuid,
	pub post_id: i64,
	pub user_id: String,
	pub content: String,
	pub create_dt: DateTime<Utc>,
}

pub struct SubThreadMessage {
	pub id: Uuid,
	pub main_thread_id: Uuid,
	pub user_id: String,
	pub content: String,
	pub create_dt: DateTime<Utc>,
}

pub struct ThreadState {
	pub room: HashMap<ThreadNumber, Chatters>,
	pub queue_client: QueueClient,
}

impl ThreadState {
	pub async fn subscribe(
		&self,
		subject: &str,
	) -> Result<Subscriber, async_nats::SubscribeError> {
		self.queue_client.subscribe(subject.to_string()).await
	}
}

impl Deref for ThreadState {
	type Target = HashMap<ThreadNumber, Chatters>;
	fn deref(&self) -> &Self::Target {
		&self.room
	}
}
impl DerefMut for ThreadState {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.room
	}
}

#[derive(Clone)]
pub struct ThreadStateWrapper(pub Arc<Mutex<ThreadState>>);
impl From<Arc<Mutex<ThreadState>>> for ThreadStateWrapper {
	fn from(value: Arc<Mutex<ThreadState>>) -> Self {
		Self(value)
	}
}
impl From<ThreadState> for ThreadStateWrapper {
	fn from(value: ThreadState) -> Self {
		Arc::new(Mutex::new(value)).into()
	}
}
impl ThreadStateWrapper {
	pub(crate) async fn write(&self) -> MutexGuard<'_, ThreadState> {
		self.0.lock().await
	}
}

// For thread on post or thread on mainthread
#[derive(Eq, Hash, PartialEq)]
pub struct ThreadNumber(i64);
impl From<i64> for ThreadNumber {
	fn from(value: i64) -> Self {
		Self(value)
	}
}

#[derive(Clone)]
pub struct Chatters(pub(crate) broadcast::Sender<String>);
impl From<broadcast::Sender<String>> for Chatters {
	fn from(value: broadcast::Sender<String>) -> Self {
		Self(value)
	}
}
impl Deref for Chatters {
	type Target = broadcast::Sender<String>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for Chatters {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
