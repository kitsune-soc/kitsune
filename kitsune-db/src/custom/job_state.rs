use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, DeriveActiveEnum, Deserialize, EnumIter, Eq, Ord, PartialEq, PartialOrd, Serialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
/// State a job can be in
pub enum JobState {
    /// Queued
    ///
    /// The job is queued for execution. It has never been executed before
    Queued = 0,
    /// Running
    ///
    /// The job is running at the moment.
    Running = 1,
    /// Failed
    ///
    /// The job has failed before. This is basically equivalent to the `Queued` state
    Failed = 2,
    /// Succeeded
    ///
    /// The job has run to completion and not errored out. The job will not be reprocessed.
    /// This entry is kept for historic purposes and can be deleted at any point in time without impacting anything.
    Succeeded = 3,
}
