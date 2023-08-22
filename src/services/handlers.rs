use std::collections::hash_map::Entry;

use axum::extract::ws::{Message, WebSocket};

use futures_util::{
	stream::{SplitSink, SplitStream},
	SinkExt, StreamExt,
};

use tokio::{
	sync::broadcast::{self},
	task::JoinHandle,
};

use crate::domain::thread::{schemas::ClientMessage, Chatters, ThreadStateWrapper};

use super::response::ServiceError;

pub struct ThreadHandler;
impl ThreadHandler {
	/// This function deals with a single websocket connection, i.e., a single
	/// connected client / user, for which we will spawn two independent tasks (for
	/// receiving / sending chat messages).
	pub async fn run_socket_broker(
		stream: WebSocket,
		state: ThreadStateWrapper,
	) {
		let (mut sender, mut receiver) = stream.split();

		let mut chatters: Option<Chatters> = None;
		while let Some(Ok(message)) = receiver.next().await {
			if let Ok(ClientMessage::JoinChat { post_id, .. }) = message.try_into() {
				chatters = Some(ThreadHandler::get_or_create_thread(post_id, state.clone()).await);
			} else {
				let _ = sender.send(String::from("Wrong input was given").into()).await;
				return;
			}
		}
		let chatters = chatters.take().unwrap();

		// Receiver from given 'Chatters' object that belongs only to the single instance

		let mut send_task = ThreadHandler::_send_messages_from_chatroom_to_this_user(chatters.clone(), sender);

		let mut recv_task = ThreadHandler::_receive_messages_from_this_user(receiver);

		// Waits on multiple concurrent branches, returning when the first branch completes,
		// cancelling the remaining branches.
		tokio::select! {
			_ = (&mut send_task) => recv_task.abort(),
			_ = (&mut recv_task) => send_task.abort(),
		};
	}

	fn _send_messages_from_chatroom_to_this_user(
		chatters: Chatters,
		mut sender: SplitSink<WebSocket, Message>,
	) -> JoinHandle<()> {
		tokio::spawn(async move {
			while let Ok(msg) = chatters.subscribe().recv().await {
				if sender.send(Message::Text(msg)).await.is_err() {
					break;
				}
			}
		})
	}
	fn _receive_messages_from_this_user(mut receiver: SplitStream<WebSocket>) -> JoinHandle<()> {
		tokio::spawn(async move {
			'recv_loop: loop {
				if let Some(Ok(message)) = receiver.next().await {
					match std::convert::TryInto::<ClientMessage>::try_into(message) {
						Ok(client_message) => {
							// TODO send client message to the topic
						}
						Err(ServiceError::UserCloseConnection) => {
							println!("User closed the connection!");
							// TODO send user_quit message to the topic
							break 'recv_loop;
						}
						_ => break 'recv_loop,
					}
				}
			}
		})
	}

	async fn get_or_create_thread(
		id: i64,
		state: ThreadStateWrapper,
	) -> Chatters {
		match state.write().await.entry(id.into()) {
			Entry::Occupied(occupied_entry) => occupied_entry.get().clone(),
			Entry::Vacant(vacant_entry) => {
				let (tx, _rx) = broadcast::channel(100);
				vacant_entry.insert(tx.clone().into());
				tx.into()
			}
		}
	}
}
