//!
//! Custom types to de-/serialise database fields with
//!
//! Something like the enum for the job states
//!

mod job_state;
mod role;
mod visibility;

pub use self::job_state::JobState;
pub use self::role::Role;
pub use self::visibility::Visibility;
