use serde::Deserialize;
use std::{borrow::Borrow, collections::HashMap, hash::Hash, marker::PhantomData};

#[derive(Debug, PartialEq)]
enum Container<V> {
    /// This case simply contains a value
    Set(V),

    /// This is set in case there was an attempt to set a value multiple times
    Overoccupied,
}

#[derive(Debug, PartialEq)]
pub struct ParamStorage<K, V>
where
    K: Eq + Hash,
{
    inner: HashMap<K, Container<V>>,
}

impl<K, V> ParamStorage<K, V>
where
    K: Eq + Hash,
{
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    #[inline]
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Eq + Hash + ?Sized,
    {
        self.inner.get(key).and_then(|container| {
            if let Container::Set(val) = container {
                Some(val)
            } else {
                None
            }
        })
    }

    #[inline]
    pub fn insert(&mut self, key: K, value: V) {
        self.inner
            .entry(key)
            .and_modify(|val| *val = Container::Overoccupied)
            .or_insert(Container::Set(value));
    }
}

impl<K, V> Default for ParamStorage<K, V>
where
    K: Eq + Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, K, V> Deserialize<'de> for ParamStorage<K, V>
where
    K: Deserialize<'de> + Eq + Hash,
    V: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor<K, V> {
            _owo: PhantomData<(K, V)>,
        }

        impl<'a, K, V> serde::de::Visitor<'a> for Visitor<K, V>
        where
            K: Deserialize<'a> + Eq + Hash,
            V: Deserialize<'a>,
        {
            type Value = ParamStorage<K, V>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a map")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'a>,
            {
                let mut collector = ParamStorage::new();
                while let Some((key, value)) = map.next_entry()? {
                    collector.insert(key, value);
                }

                Ok(collector)
            }
        }

        deserializer.deserialize_map(Visitor { _owo: PhantomData })
    }
}
