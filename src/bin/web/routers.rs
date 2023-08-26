use axum::{
	extract::{State, WebSocketUpgrade},
	headers::{self, authorization::Bearer, Authorization},
	response::IntoResponse,
	routing::get,
	Router, TypedHeader,
};

use post::{domain::thread::ThreadStateWrapper, services::handlers::ThreadHandler};

async fn chat_websocket_route(
	ws: WebSocketUpgrade,
	// TODO should be replacted with middleware
	current_user: Option<TypedHeader<headers::Authorization<Bearer>>>,
	State(state): State<ThreadStateWrapper>,
) -> impl IntoResponse {
	let token = if let Some(TypedHeader(Authorization::<Bearer>(value))) = current_user {
		value.token().to_string()
	} else {
		tracing::info!("Unknown browser Accessed!");
		String::from("Unknown browser")
	};

	// TODO jollidah -> Access layer need to be added
	println!("User Access Token: `{token:?}` ???");

	ws.on_upgrade(|socket| ThreadHandler::run_socket_broker(socket, state))
}

// ! deprecated
use axum::response::Html;
async fn index() -> Html<&'static str> {
	Html(std::include_str!("./chat.html"))
}

pub fn post() -> Router<ThreadStateWrapper> {
	let router = Router::new().route("/chat/ws", get(chat_websocket_route));

	router.route("/chat", get(index))
}
