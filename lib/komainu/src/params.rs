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
    pub(crate) fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

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

    fn insert(&mut self, key: K, value: V) {
        self.inner
            .entry(key)
            .and_modify(|val| *val = Container::Overoccupied)
            .or_insert(Container::Set(value));
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

#[cfg(test)]
mod test {
    use serde_test::Token;

    use super::ParamStorage;

    #[test]
    fn insert_get_works() {
        let mut map = ParamStorage::new();
        map.insert("hello", "world");
        assert_eq!(map.get("hello"), Some(&"world"));
    }

    #[test]
    fn multi_insert_empty() {
        let mut map = ParamStorage::new();
        map.insert("hello", "world");
        map.insert("hello", "owo");
        assert_eq!(map.get("hello"), None);

        map.insert("hello", "uwu");
        assert_eq!(map.get("hello"), None);
    }

    #[test]
    fn deserialize_impl() {
        let mut map1 = ParamStorage::new();
        map1.insert("hello", "world");

        serde_test::assert_de_tokens(
            &map1,
            &[
                Token::Map { len: Some(1) },
                Token::BorrowedStr("hello"),
                Token::BorrowedStr("world"),
                Token::MapEnd,
            ],
        );

        let mut map2 = ParamStorage::new();
        map2.insert("hello", "world");
        map2.insert("hello", "owo");

        serde_test::assert_de_tokens(
            &map2,
            &[
                Token::Map { len: Some(2) },
                Token::BorrowedStr("hello"),
                Token::BorrowedStr("world"),
                Token::BorrowedStr("hello"),
                Token::BorrowedStr("owo"),
                Token::MapEnd,
            ],
        );
    }
}
