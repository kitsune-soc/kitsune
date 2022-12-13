use sea_orm::prelude::*;

#[derive(Clone, Debug, DeriveEntityModel, Eq, PartialEq)]
#[sea_orm(table_name = "posts_mentions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub post_id: Uuid,
    #[sea_orm(primary_key, auto_increment = false)]
    pub user_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
