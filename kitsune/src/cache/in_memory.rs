use super::{Cache, CacheResult};
use async_trait::async_trait;
use dashmap::DashMap;
use std::{
    fmt::Display,
    marker::PhantomData,
    time::{Duration, Instant},
};

struct CacheValue<V> {
    created_at: Instant,
    value: V,
}

pub struct InMemoryCache<K, V>
where
    K: ?Sized,
{
    inner: DashMap<String, CacheValue<V>>,
    size: usize,
    ttl: Duration,
    _key_ty: PhantomData<K>,
}

impl<K, V> InMemoryCache<K, V>
where
    K: Display + ?Sized,
{
    #[must_use]
    pub fn new(size: usize, ttl: Duration) -> Self {
        Self {
            inner: DashMap::with_capacity(size),
            size,
            ttl,
            _key_ty: PhantomData,
        }
    }
}

#[async_trait]
impl<K, V> Cache<K, V> for InMemoryCache<K, V>
where
    K: Display + Send + Sync + ?Sized,
    V: Clone + Send + Sync,
{
    async fn delete(&self, key: &K) -> CacheResult<()> {
        self.inner.remove(&key.to_string());
        Ok(())
    }

    async fn get(&self, key: &K) -> CacheResult<Option<V>> {
        let key = key.to_string();
        if let Some(value) = self.inner.get(&key) {
            if value.created_at.elapsed() > self.ttl {
                drop(value); // Load bearing drop. Otherwise call will deadlock.
                self.inner.remove(&key);
                return Ok(None);
            }

            return Ok(Some(value.value.clone()));
        }

        Ok(None)
    }

    async fn set(&self, key: &K, value: &V) -> CacheResult<()> {
        if self.inner.len() == self.size {
            let first_key = {
                let first_entry = self.inner.iter().next().unwrap();
                first_entry.key().clone()
            };

            self.inner.remove(&first_key);
        }

        self.inner.insert(
            key.to_string(),
            CacheValue {
                created_at: Instant::now(),
                value: value.clone(),
            },
        );

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::cache::{Cache, InMemoryCache};
    use std::time::Duration;

    #[tokio::test]
    async fn delete_expired_key() {
        let cache = InMemoryCache::new(10, Duration::from_millis(10));
        cache.set(&"hello", &"world").await.unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(cache.get(&"hello").await.unwrap(), None);
    }

    #[tokio::test]
    async fn put_into_full_cache() {
        let cache = InMemoryCache::new(1, Duration::from_secs(60));
        cache.set(&"hello", &"world").await.unwrap();
        cache.set(&"another", &"pair").await.unwrap();

        assert_eq!(cache.inner.len(), 1);
    }
}
