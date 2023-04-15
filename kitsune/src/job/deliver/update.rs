use crate::{
    error::Result,
    job::{JobContext, Runnable},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Deserialize, Serialize)]
pub enum UpdateEntity {
    Account,
    Status,
}

#[derive(Deserialize, Serialize)]
pub struct DeliverUpdate {
    entity: UpdateEntity,
    id: Uuid,
}

#[async_trait]
impl Runnable for DeliverUpdate {
    async fn run(&self, context: JobContext<'_>) -> Result<()> {
        todo!();
    }
}
