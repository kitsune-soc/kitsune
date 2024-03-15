use crate::container::Service;
use std::{borrow::Cow, sync::OnceLock};

pub static CONTAINER_CLIENT: OnceLock<testcontainers::clients::Cli> = OnceLock::new();

#[macro_export]
macro_rules! get_resource {
    ($env_name:literal, $container_fn:path) => {
        ::std::env::var($env_name).map_or_else(
            |_| {
                // Only initialize client if we actually need it
                let client = $crate::resource::CONTAINER_CLIENT.get_or_init(|| {
                    ::testcontainers::clients::Cli::new::<::testcontainers::core::env::Os>()
                });

                $crate::resource::ResourceHandle::Container($container_fn(client))
            },
            $crate::resource::ResourceHandle::Url,
        )
    };
}

pub enum ResourceHandle<S>
where
    S: Service,
{
    Container(S),
    Url(String),
}

impl<S> ResourceHandle<S>
where
    S: Service,
{
    pub fn url(&self) -> Cow<'_, str> {
        match self {
            Self::Container(container) => Cow::Owned(container.url()),
            Self::Url(ref url) => Cow::Borrowed(url),
        }
    }
}