use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

pub trait Strategy<Object>: Send + Sync {
    fn select<'a>(&self, obj: &'a [Object]) -> &'a Object;
}

#[derive(Default)]
pub struct RoundRobinStrategy {
    counter: AtomicUsize,
}

impl<Object> Strategy<Object> for RoundRobinStrategy {
    fn select<'a>(&self, objects: &'a [Object]) -> &'a Object {
        let count = self.counter.fetch_add(1, Ordering::AcqRel);
        &objects[count % objects.len()]
    }
}

struct Inner<Object> {
    strategy: Box<dyn Strategy<Object>>,
    objects: Box<[Object]>,
}

pub struct Pool<Object> {
    inner: Arc<Inner<Object>>,
}

impl<Object> Pool<Object> {
    pub async fn from_producer<P, Fut, Err, S>(
        mut producer: P,
        count: usize,
        strategy: S,
    ) -> Result<Self, Err>
    where
        P: FnMut() -> Fut,
        Fut: Future<Output = Result<Object, Err>>,
        S: Strategy<Object> + 'static,
    {
        let mut objects = Vec::with_capacity(count);
        for _ in 0..count {
            objects.push((producer)().await?);
        }

        Ok(Self::new(objects, strategy))
    }

    pub fn new<O, S>(objects: O, strategy: S) -> Self
    where
        O: Into<Box<[Object]>>,
        S: Strategy<Object> + 'static,
    {
        Self {
            inner: Arc::new(Inner {
                strategy: Box::new(strategy),
                objects: objects.into(),
            }),
        }
    }

    #[must_use]
    pub fn get_ref(&self) -> &Object {
        self.inner.strategy.select(&self.inner.objects)
    }
}

impl<Object> Pool<Object>
where
    Object: Clone,
{
    #[must_use]
    pub fn get(&self) -> Object {
        self.get_ref().clone()
    }
}

impl<Object> Clone for Pool<Object> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Pool, RoundRobinStrategy};

    #[test]
    fn test_round_robin() {
        let pool = Pool::new((0..10).collect::<Vec<_>>(), RoundRobinStrategy::default());

        for cnt in 0..20 {
            let item = pool.get();
            assert_eq!(item, cnt % 10);
        }
    }
}
