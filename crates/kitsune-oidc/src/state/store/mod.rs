use super::LoginState;
use crate::error::Result;
use enum_dispatch::enum_dispatch;

pub use self::{in_memory::InMemory, redis::Redis};

pub mod in_memory;
pub mod redis;

#[derive(Clone)]
#[enum_dispatch(Store)]
pub enum StoreBackend {
    InMemory(in_memory::InMemory),
    Redis(redis::Redis),
}

#[enum_dispatch]
pub trait Store {
    async fn get_and_remove(&self, key: &str) -> Result<LoginState>;
    async fn set(&self, key: &str, value: LoginState) -> Result<()>;
}
