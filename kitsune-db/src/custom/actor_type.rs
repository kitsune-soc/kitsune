use kitsune_type::ap::actor::ActorType as ApActorType;
use sea_orm::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(
    Clone, Debug, DeriveActiveEnum, Deserialize, EnumIter, Eq, Ord, PartialEq, PartialOrd, Serialize,
)]
#[sea_orm(rs_type = "i32", db_type = "Integer")]
/// ActivityPub actor types
pub enum ActorType {
    /// Actor representing a group
    Group = 0,

    /// Acotr representing an individual
    Person = 1,
}

impl From<ApActorType> for ActorType {
    fn from(value: ApActorType) -> Self {
        match value {
            ApActorType::Group => Self::Group,
            ApActorType::Person => Self::Person,
        }
    }
}

impl From<ActorType> for ApActorType {
    fn from(value: ActorType) -> Self {
        match value {
            ActorType::Group => Self::Group,
            ActorType::Person => Self::Person,
        }
    }
}
