use event_driven_library::{repository::TRepository, Aggregate};

use crate::{domain::post::PostAggregate, services::response::ServiceError};

use super::Repository;

impl Repository<PostAggregate> {
	pub async fn get(&self) -> Result<PostAggregate, ServiceError> {
		todo!()
	}

	pub async fn update(
		&mut self,
		aggregate: &mut PostAggregate,
	) -> Result<(), ServiceError> {
		self.set_events(aggregate.collect_events());
		todo!()
	}
}
