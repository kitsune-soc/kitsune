use crate::{
    error::Result,
    job::{JobContext, Runnable},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DeliverUpdate {}

#[async_trait]
impl Runnable for DeliverUpdate {
    async fn run(&self, context: JobContext<'_>) -> Result<()> {
        todo!();
    }
}
