use std::ops::{Deref, DerefMut};

use self::entity::{Post, PostContent};
use event_driven_library::{prelude::*, Entity};
pub mod commands;
pub mod entity;
pub mod events;

#[derive(Default, Debug, Serialize, Deserialize, AggregateMacro!, Entity!)]
pub struct PostAggregate {
	#[serde(skip_deserializing, skip_serializing)]
	events: std::collections::VecDeque<std::boxed::Box<dyn Message>>,
	pub(crate) post: Post,
	pub(crate) content: PostContent,
	pub(crate) like: i64,
}

impl Deref for PostAggregate {
	type Target = Post;
	fn deref(&self) -> &Self::Target {
		&self.post
	}
}
impl DerefMut for PostAggregate {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.post
	}
}
