use std::sync::OnceLock;

use async_trait::async_trait;
use event_driven_library::{repository::Executor, responses::BaseError};

use mongodb::Client;

#[derive(Debug)]
pub struct ServiceExecutor {
	/// because Client does its own pooling internally, attempting to maintain a pool of Clients will
	/// (somewhat counter-intuitively) result in worse performance than using a single one.
	client: &'static Client,
}

impl ServiceExecutor {
	pub async fn new() -> Self {
		Self {
			client: mongo_client().await,
		}
	}
}

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

pub async fn mongo_client() -> &'static Client {
	static CLIENT: OnceLock<Client> = OnceLock::new();
	dotenv::dotenv().ok();

	let p = match CLIENT.get() {
		None => {
			let url = &std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

			let client = Client::with_uri_str(&url).await.unwrap();
			CLIENT.get_or_init(|| client)
		}
		Some(client) => client,
	};
	p
}

#[cfg(test)]
mod test {
	use mongodb::{bson::doc, Client};

	use crate::database::mongo_client;
	use futures::stream::TryStreamExt;

	async fn mongo_cli() -> &'static Client {
		let client = mongo_client().await;
		client
	}

	pub(crate) async fn set_up(db_name: &str) {
		let client = mongo_cli().await;
		let _ = client.database(db_name).drop(None).await;
	}
	#[tokio::test]
	async fn test_mongo_client() {
		let client = mongo_cli().await;
		let db = client.database("mydb");

		for db_name in db.list_collection_names(None).await.unwrap() {
			println!("{}", db_name);
		}
	}

	#[tokio::test]
	async fn test_insert() {
		'_given: {
			let db_name = "test_db";
			let collection_name = "test_collection";

			set_up(db_name).await;

			let cl = mongo_cli().await;
			let collection = cl.database(db_name).collection(collection_name);

			'_when: {
				collection
					.insert_one(doc! { "title": "1984", "author": "George Orwell" }, None)
					.await
					.unwrap();

				'_then: {
					let filter = doc! { "author": "George Orwell" };
					let mut cursor = collection.find(filter, None).await.unwrap();
					let Some(res) = cursor.try_next().await.unwrap() else{
						panic!("Nothing returned!")
					};

					println!("{:?}", res)
				}
			}
		}
	}
}
