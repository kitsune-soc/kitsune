use super::{Id, Set};
use core::marker::PhantomData;
use serde::de::{Deserialize, DeserializeSeed, Deserializer};

/// Deserialises a JSON-LD set of nodes as a sequence of node identifier strings.
pub struct IdSet<T> {
    seed: T,
}

impl<'de, T> IdSet<PhantomData<T>>
where
    T: Deserialize<'de>,
{
    pub fn new() -> Self {
        Self::with_seed(PhantomData)
    }

    pub fn deserialize<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new().deserialize(deserializer)
    }
}

impl<'de, T> IdSet<T>
where
    T: DeserializeSeed<'de>,
{
    pub fn with_seed(seed: T) -> Self {
        Self { seed }
    }
}

impl<'de, T> Default for IdSet<PhantomData<T>>
where
    T: Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, T> DeserializeSeed<'de> for IdSet<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Set::with_seed(Id::with_seed(self.seed)).deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::into_deserializer, IdSet};
    use serde::Deserialize;
    use serde_test::{assert_de_tokens, Token};
    use std::collections::HashMap;

    #[test]
    fn single_string() {
        let data = "http://example.com/";
        assert_eq!(
            IdSet::deserialize(into_deserializer(data)),
            Ok(vec![data.to_owned()])
        );
    }

    #[test]
    fn single_embedded() {
        let data: HashMap<_, _> = [("id", "http://example.com/")].into_iter().collect();
        assert_eq!(
            IdSet::deserialize(into_deserializer(data)),
            Ok(vec!["http://example.com/".to_owned()])
        );
    }

    #[test]
    fn seq() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(transparent)]
        struct Test {
            #[serde(deserialize_with = "IdSet::deserialize")]
            term: Vec<String>,
        }

        assert_de_tokens(
            &Test {
                term: vec![
                    "http://example.com/1".to_owned(),
                    "http://example.com/2".to_owned(),
                ],
            },
            &[
                Token::Seq { len: Some(2) },
                Token::Str("http://example.com/1"),
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("http://example.com/2"),
                Token::MapEnd,
                Token::SeqEnd,
            ],
        );
    }
}
