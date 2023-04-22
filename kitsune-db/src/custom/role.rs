use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone,
    Copy,
    Debug,
    DeriveActiveEnum,
    Deserialize,
    EnumIter,
    Eq,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
/// Role of a local user on this server
pub enum Role {
    /// Administrator
    ///
    /// This user is an administrator on this instance and has elevated access
    Administrator = 0,
}
