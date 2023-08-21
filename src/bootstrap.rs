use std::{
	any::{Any, TypeId},
	collections::HashMap,
	sync::OnceLock,
};

use event_driven_library::{
	domain::Message,
	init_command_handler, init_event_handler,
	messagebus::AtomicContextManager,
	messagebus::{Future, MessageBus},
};

use crate::services::response::ServiceError;

use crate::dependencies::dependency;
use crate::services::response::ServiceResponse;

pub type TEventHandler<T> = HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, T) -> Future<ServiceResponse, ServiceError> + Send + Sync>>>;
pub type TCommandHandler<T> = HashMap<TypeId, Box<dyn Fn(Box<dyn Any + Send + Sync>, T) -> Future<ServiceResponse, ServiceError> + Send + Sync>>;

pub struct Boostrap;
impl Boostrap {
	pub async fn message_bus() -> std::sync::Arc<MessageBus<ServiceResponse, ServiceError>> {
		MessageBus::new(command_handler().await, event_handler().await)
	}
}

///* `Dependency` is the struct you implement injectable dependencies that will be called inside the function.

#[cfg(test)]
pub static EVENT_COUNTER: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(0);

// * Among dependencies, `Connectable` dependencies shouldn't be injected sometimes because
// * its state is usually globally managed as in conneciton pool in RDBMS.
// * Therefore, it's adviable to specify connectables seperately.
init_command_handler!({});

init_event_handler!({});

/// static values

pub async fn command_handler() -> &'static TCommandHandler<AtomicContextManager> {
	static COMMAND_HANDLER: OnceLock<TCommandHandler<AtomicContextManager>> = OnceLock::new();
	let ch = match COMMAND_HANDLER.get() {
		None => {
			let command_handler = init_command_handler().await;

			COMMAND_HANDLER.get_or_init(|| command_handler)
		}
		Some(command_handler) => command_handler,
	};
	ch
}

pub async fn event_handler() -> &'static TEventHandler<AtomicContextManager> {
	static EVENT_HANDLER: OnceLock<TEventHandler<AtomicContextManager>> = OnceLock::new();
	let eh = match EVENT_HANDLER.get() {
		None => {
			let event_handler = init_event_handler().await;

			EVENT_HANDLER.get_or_init(|| event_handler)
		}
		Some(event_handler) => event_handler,
	};
	eh
}
