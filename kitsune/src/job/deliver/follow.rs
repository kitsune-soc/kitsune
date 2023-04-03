use crate::{
    error::Result,
    job::{JobContext, Runnable},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DeliverFollow {}

#[async_trait]
impl Runnable for DeliverFollow {
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        todo!();
    }
}
