use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

trait Strategy<Object>: Send + Sync {
    fn select<'a>(&self, obj: &'a [Object]) -> &'a Object;
}

#[derive(Default)]
struct RoundRobinStrategy {
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

impl<Object> FromIterator<Object> for Pool<Object> {
    fn from_iter<T: IntoIterator<Item = Object>>(iter: T) -> Self {
        Self {
            inner: Arc::new(Inner {
                strategy: Box::<RoundRobinStrategy>::default(),
                objects: iter.into_iter().collect(),
            }),
        }
    }
}

impl<Object> Pool<Object>
where
    Object: Clone,
{
    #[must_use]
    pub fn get(&self) -> Object {
        let selected = self.inner.strategy.select(&self.inner.objects);
        selected.clone()
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
    use crate::Pool;

    #[test]
    fn test_round_robin() {
        let pool = (0..10).collect::<Pool<_>>();

        for cnt in 0..20 {
            let item = pool.get();
            assert_eq!(item, cnt % 10);
        }
    }
}
