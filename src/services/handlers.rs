use std::{collections::hash_map::Entry, pin::Pin};

use async_nats::jetstream::{self, consumer::PullConsumer};
use axum::extract::ws::{Message, WebSocket};

use futures::{
	stream::{SplitSink, SplitStream},
	SinkExt, StreamExt, TryStreamExt,
};

use tokio::{
	sync::broadcast::{self},
	task::JoinHandle,
};

use crate::{
	dependencies::queue_client,
	domain::thread::{schemas::ClientMessage, Chatters, ThreadStateWrapper},
};

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
			if let Ok(ClientMessage::JoinChat { post_id, user_id }) = message.try_into() {
				chatters = Some(ThreadHandler::get_or_create_thread(post_id, state.clone()).await);

				// Notify user join event by publishing it to a queue service
				if let Err(ServiceError::MessagePublishingError) =
					ThreadHandler::publish_client_message(ClientMessage::JoinChat { post_id, user_id }, state.clone()).await
				{
					let _ = sender.send(String::from("Message Publishing failed!").into()).await;
					return;
				}
			} else {
				let _ = sender.send(String::from("Wrong input was given").into()).await;
				return;
			}
		}
		let chatters = chatters.take().unwrap();

		// Receiver from given 'Chatters' object that belongs only to the single instance

		let mut send_task = ThreadHandler::_send_messages_from_chatroom_to_this_user(chatters.clone(), sender);

		let mut recv_task = ThreadHandler::_receive_messages_from_this_user(receiver, state.clone());

		// Waits on multiple concurrent branches, returning when the first branch completes,
		// cancelling the remaining branches.
		tokio::select! {
			_ = (&mut send_task) => recv_task.abort(),
			_ = (&mut recv_task) => send_task.abort(),
		};
	}
	pub async fn publish_client_message(
		msg: ClientMessage,
		state: ThreadStateWrapper,
	) -> Result<(), ServiceError> {
		// TODO Send client message

		state.write().await.queue_client.send(msg.subject(), msg).await.map_err(|err| {
			println!("{:?}", err);
			tracing::error!("Message publishing error while notifying user join :{:?}", err);
			ServiceError::MessagePublishingError
		})?;

		Ok(())
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
	fn _receive_messages_from_this_user(
		mut receiver: SplitStream<WebSocket>,
		state: ThreadStateWrapper,
	) -> JoinHandle<Result<(), ServiceError>> {
		tokio::spawn(async move {
			loop {
				if let Some(Ok(message)) = receiver.next().await {
					let client_message = std::convert::TryInto::<ClientMessage>::try_into(message)?;
					ThreadHandler::publish_client_message(client_message, state.clone()).await?;
				}
			}
		})
	}

	async fn get_consumer(
		state: ThreadStateWrapper,
		durable_name: &str,
	) -> Result<PullConsumer, ServiceError> {
		// TODO subjects from config
		// TODO error handling

		let consumer = state
			.write()
			.await
			.queue_client
			.get_or_create_stream("chat_stream", vec!["chat.>".to_string()])
			.await
			.map_err(|err| {
				println!("First Error point : {}", err);
				ServiceError::QueueServiceError
			})?
			.get_or_create_consumer(
				"chat_consumer",
				jetstream::consumer::pull::Config {
					// inactive_threshold: Duration::from_secs(60),
					// TODO Set durable group
					durable_name: Some(durable_name.into()),

					..Default::default()
				},
			)
			.await
			.map_err(|err| {
				println!("second Error point{}", err);
				ServiceError::QueueServiceError
			})?;
		Ok(consumer)
	}
	async fn run_global_consumer(consumer: PullConsumer) -> Result<(), ServiceError> {
		let mut batches = consumer.sequence(10).map_err(|_| ServiceError::QueueServiceError)?.take(10);

		'batch_processing: {
			while let Ok(Some(mut batch)) = batches.try_next().await {
				println!("Whatttup");
				while let Ok(Some(message)) = batch.try_next().await {
					let clinet_message =
						serde_json::from_str::<ClientMessage>(std::str::from_utf8(&message.payload).map_err(|_| ServiceError::ParsingError)?)
							.map_err(|_| ServiceError::ParsingError)?;
					// TODO distributing messages to
					match clinet_message {
						ClientMessage::JoinChat { post_id, user_id } => println!("Got Joint Chat"),
						ClientMessage::WriteMainThread { post_id, user_id, content } => println!("Got WriteMainThread"),
						ClientMessage::WriteSubThread {
							main_thread_id,
							user_id,
							content,
						} => println!("Got WriteSubThread"),
					}

					// acknowledge the message
					message.ack().await.unwrap();

					if cfg!(test) {
						break 'batch_processing;
					}
				}
			}
		}
		Ok(())
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

#[cfg(test)]
mod test {

	use async_nats::jetstream::consumer::PullConsumer;
	use chrono::Duration;
	use futures::StreamExt;
	use futures::TryStreamExt;
	use rand::Rng;

	use crate::{
		dependencies::queue_client,
		domain::thread::{schemas::ClientMessage, ThreadState, ThreadStateWrapper},
		services::handlers::ThreadHandler,
	};
	async fn set_up(state: ThreadStateWrapper) {
		state
			.write()
			.await
			.queue_client
			.get_stream("chat_stream")
			.await
			.map_err(|err| {
				println!("{}", err);
				err
			})
			.unwrap()
			.purge()
			.filter("chat.>")
			.await
			.unwrap();
	}

	#[tokio::test]
	async fn test_notify_user_join_event() {
		'_given: {
			let durable_name = "test_durable";
			let chat_state: ThreadStateWrapper = ThreadState {
				room: Default::default(),
				queue_client: queue_client().await.into(),
			}
			.into();

			set_up(chat_state.clone()).await;

			'_when: {
				let mut rng = rand::thread_rng();
				let p_id = rng.gen::<i64>();
				println!("{}", p_id);

				// Notify user join event
				if let Err(err) = ThreadHandler::publish_client_message(
					ClientMessage::JoinChat {
						post_id: p_id,
						user_id: "MigoMigo".to_string(),
					},
					chat_state.clone(),
				)
				.await
				{
					panic!("Error! {:?}", err)
				}

				let consumer: PullConsumer = ThreadHandler::get_consumer(chat_state.clone(), durable_name).await.unwrap();

				//TODO Hang!
				println!("Hang?");
				if let Ok(Some(msg)) = consumer.messages().await.unwrap().take(1).try_next().await {
					println!("Got Message!");
					let message = serde_json::from_str::<ClientMessage>(std::str::from_utf8(&msg.payload).unwrap()).unwrap();
					if let ClientMessage::JoinChat { post_id, user_id } = message {
						assert_eq!(post_id, p_id);
						assert_eq!(user_id, "MigoMigo".to_string(),);
					};
					msg.ack().await.unwrap();
				}

				// TODO consume that message
			}
		}
	}

	#[tokio::test]
	async fn test_run_global_consumer() {
		'_given: {
			let durable_name = "test_durable";
			let chat_state: ThreadStateWrapper = ThreadState {
				room: Default::default(),
				queue_client: queue_client().await.into(),
			}
			.into();
			set_up(chat_state.clone()).await;
			let consumer = ThreadHandler::get_consumer(chat_state.clone(), durable_name).await.unwrap();

			if let Err(err) = ThreadHandler::publish_client_message(
				ClientMessage::JoinChat {
					post_id: 2,
					user_id: "MagoMago".to_string(),
				},
				chat_state.clone(),
			)
			.await
			{
				panic!("Error! {:?}", err)
			}

			ThreadHandler::run_global_consumer(consumer).await.unwrap();
		}
	}
}
