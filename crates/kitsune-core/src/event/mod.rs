use kitsune_messaging::{MessageConsumer, MessageEmitter};

pub use self::post::PostEvent;

pub mod post;

pub type PostEventConsumer = MessageConsumer<PostEvent>;
pub type PostEventEmitter = MessageEmitter<PostEvent>;
