use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::services::response::ServiceError;

#[derive(Debug, Deserialize, Serialize)]
pub enum ClientMessage {
	JoinChat {
		post_id: i64,
		user_id: String,
	},
	WriteMainThread {
		post_id: i64,
		user_id: String,
		content: String,
	},
	WriteSubThread {
		main_thread_id: Uuid,
		user_id: String,
		content: String,
	},
}

impl TryFrom<axum::extract::ws::Message> for ClientMessage {
	type Error = ServiceError;
	fn try_from(value: axum::extract::ws::Message) -> Result<Self, Self::Error> {
		match value {
			axum::extract::ws::Message::Text(string_value) => {
				serde_json::from_str::<ClientMessage>(&string_value).map_err(|_err| ServiceError::ParsingError)
			}

			axum::extract::ws::Message::Close(_close_frame) => Err(ServiceError::UserCloseConnection),
			_ => Err(ServiceError::BadRequest),
		}
	}
}

#[test]
fn test_enum_representation() {
	let join_message = ClientMessage::JoinChat {
		post_id: 1,
		user_id: "Migo".to_string(),
	};

	let jsonified = serde_json::to_string(&join_message).unwrap();
	println!("{:?}", jsonified);

	println!("{:?}", serde_json::from_str::<ClientMessage>(&jsonified).unwrap());
}
