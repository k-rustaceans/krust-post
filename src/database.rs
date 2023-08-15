use async_trait::async_trait;
use event_driven_library::{repository::Executor, responses::BaseError};

#[derive(Debug)]
pub struct ServiceExecutor {}

#[async_trait]
impl Executor for ServiceExecutor {
	async fn begin(&mut self) -> Result<(), BaseError> {
		todo!()
	}
	async fn commit(&mut self) -> Result<(), BaseError> {
		todo!()
	}
	async fn rollback(&mut self) -> Result<(), BaseError> {
		todo!()
	}
}
