use crate::{
    error::Result,
    job::{JobContext, Runnable},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct DeliverUnfollow {}

#[async_trait]
impl Runnable for DeliverUnfollow {
    async fn run(&self, ctx: JobContext<'_>) -> Result<()> {
        todo!();
    }
}
