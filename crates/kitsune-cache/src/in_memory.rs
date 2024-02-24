use crate::{CacheBackend, CacheResult};
use moka::future::Cache;
use std::{fmt::Display, marker::PhantomData, time::Duration};

pub struct InMemory<K, V>
where
    K: ?Sized,
{
    inner: Cache<String, V>,
    _key_type: PhantomData<K>,
}

impl<K, V> InMemory<K, V>
where
    K: Display + ?Sized,
    V: Clone + Send + Sync + 'static,
{
    #[must_use]
    pub fn new(size: usize, ttl: Duration) -> Self {
        let inner = Cache::builder()
            .time_to_live(ttl)
            .max_capacity(size as u64)
            .build();

        Self {
            inner,
            _key_type: PhantomData,
        }
    }
}

impl<K, V> CacheBackend<K, V> for InMemory<K, V>
where
    K: Display + Send + Sync + ?Sized,
    V: Clone + Send + Sync + 'static,
{
    async fn delete(&self, key: &K) -> CacheResult<()> {
        self.inner.remove(&key.to_string()).await;
        Ok(())
    }

    async fn get(&self, key: &K) -> CacheResult<Option<V>> {
        Ok(self.inner.get(&key.to_string()).await)
    }

    async fn set(&self, key: &K, value: &V) -> CacheResult<()> {
        self.inner.insert(key.to_string(), value.clone()).await;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{CacheBackend, InMemoryCache};
    use std::time::Duration;

    #[tokio::test]
    async fn delete_expired_key() {
        let cache = InMemoryCache::new(10, Duration::from_millis(10));
        cache.set(&"hello", &"world").await.unwrap();
        assert_eq!(cache.get(&"hello").await.unwrap(), Some("world"));
        tokio::time::sleep(Duration::from_millis(100)).await;
        assert_eq!(cache.get(&"hello").await.unwrap(), None);
    }

    #[tokio::test]
    async fn put_into_full_cache() {
        let cache = InMemoryCache::new(1, Duration::from_secs(60));
        cache.set(&"hello", &"world").await.unwrap();
        cache.set(&"another", &"pair").await.unwrap();

        cache.inner.run_pending_tasks().await;

        assert_eq!(cache.inner.entry_count(), 1);
    }
}
