use axum::extract::ws::{Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast::{self, Sender};

use crate::domain::thread::{schemas::ClientMessage, Chatters, ThreadStateWrapper};

pub(crate) struct ThreadHandler;
impl ThreadHandler {
	/// This function deals with a single websocket connection, i.e., a single
	/// connected client / user, for which we will spawn two independent tasks (for
	/// receiving / sending chat messages).
	async fn handle_socket(
		stream: WebSocket,
		state: ThreadStateWrapper,
	) {
		let (mut sender, mut receiver) = stream.split();

		let mut chatters: Option<Chatters> = None;
		while let Some(Ok(message)) = receiver.next().await {
			if let Ok(ClientMessage::JoinMainThread { post_id, .. }) = message.try_into() {
				chatters = Some(ThreadHandler::get_or_create_thread(post_id, state.clone()).await);
			} else {
				let _ = sender
					.send(String::from("Wrong input was given").into())
					.await;
			}
		}
		let chatters = chatters.take().unwrap();

		// Receiver from given 'Chatters' object that belongs only to the single instance
		let mut rx = chatters.subscribe();

		// TODO Send the join message to queue

		// TODO take message from either queue(pulsar) or chatters in the given application instance
	}

	async fn get_or_create_thread(
		post_id: i64,
		state: ThreadStateWrapper,
	) -> Chatters {
		let (tx, _rx) = broadcast::channel(100);

		state
			.write()
			.await
			.entry(post_id.into())
			.or_insert(tx.clone().into());
		tx.into()
	}
}
