//!
//! Custom types to de-/serialise database fields with
//!
//! Something like the enum for the job states
//!

mod actor_type;
mod job_state;
mod role;
mod visibility;

pub mod tsvector_column;

pub use self::actor_type::ActorType;
pub use self::job_state::JobState;
pub use self::role::Role;
pub use self::visibility::Visibility;
