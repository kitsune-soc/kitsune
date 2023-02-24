use kitsune_messaging::{MessageConsumer, MessageEmitter};

pub use self::status::StatusEvent;

pub mod status;

pub type StatusEventConsumer = MessageConsumer<StatusEvent>;
pub type StatusEventEmitter = MessageEmitter<StatusEvent>;
