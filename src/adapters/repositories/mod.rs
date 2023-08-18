pub(crate) mod post_repository;
use std::mem;
use std::{collections::VecDeque, marker::PhantomData, sync::Arc};

use event_driven_library::domain::Aggregate;

use event_driven_library::domain::Message;
use event_driven_library::repository::TRepository;
use tokio::sync::RwLock;

use crate::database::ServiceExecutor;

pub struct Repository<A: Aggregate> {
	pub executor: Arc<RwLock<ServiceExecutor>>,
	pub _phantom: PhantomData<A>,
	pub events: VecDeque<Box<dyn Message>>,
}

impl<A: Aggregate + 'static> TRepository<ServiceExecutor> for Repository<A> {
	fn new(executor: Arc<RwLock<ServiceExecutor>>) -> Self {
		Self {
			executor,
			_phantom: Default::default(),
			events: Default::default(),
		}
	}
	fn get_events(&mut self) -> VecDeque<Box<dyn Message>> {
		mem::take(&mut self.events)
	}
	fn set_events(
		&mut self,
		events: VecDeque<Box<dyn Message>>,
	) {
		self.events = events
	}
}
