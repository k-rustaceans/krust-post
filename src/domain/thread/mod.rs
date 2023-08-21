pub mod schemas;
use std::{
	collections::HashMap,
	ops::{Deref, DerefMut},
	sync::Arc,
};

// domain for messaging feature
use chrono::{DateTime, Utc};
use tokio::sync::{broadcast, Mutex, MutexGuard};
use uuid::Uuid;

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

pub struct ThreadState(HashMap<ThreadNumber, Chatters>);

impl Deref for ThreadState {
	type Target = HashMap<ThreadNumber, Chatters>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl DerefMut for ThreadState {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

#[derive(Clone)]
pub struct ThreadStateWrapper(Arc<Mutex<ThreadState>>);
impl From<Arc<Mutex<ThreadState>>> for ThreadStateWrapper {
	fn from(value: Arc<Mutex<ThreadState>>) -> Self {
		Self(value)
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
