use super::{First, Id};
use core::marker::PhantomData;
use serde::de::{Deserialize, DeserializeSeed, Deserializer};

/// Deserialises the node identifier string of the first element of a JSON-LD set.
pub struct FirstId<T> {
    seed: T,
}

impl<'de, T> FirstId<PhantomData<T>>
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

impl<'de, T> FirstId<T>
where
    T: DeserializeSeed<'de>,
{
    pub fn with_seed(seed: T) -> Self {
        Self { seed }
    }
}

impl<'de, T> DeserializeSeed<'de> for FirstId<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        First::with_seed(Id::with_seed(self.seed)).deserialize(deserializer)
    }
}

#[cfg(test)]
mod tests {
    use super::{super::into_deserializer, FirstId};
    use serde::Deserialize;
    use serde_test::{assert_de_tokens, Token};
    use std::collections::HashMap;

    #[test]
    fn single_string() {
        let data = "http://example.com/".to_owned();
        assert_eq!(FirstId::deserialize(into_deserializer(&data[..])), Ok(data));
    }

    #[test]
    fn single_embedded() {
        let data: HashMap<_, _> = [("id", "http://example.com/")].into_iter().collect();
        assert_eq!(
            FirstId::deserialize(into_deserializer(data)),
            Ok("http://example.com/".to_owned())
        );
    }

    #[test]
    fn seq() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(transparent)]
        struct Test {
            #[serde(deserialize_with = "FirstId::deserialize")]
            term: String,
        }

        assert_de_tokens(
            &Test {
                term: "http://example.com/1".to_owned(),
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

        assert_de_tokens(
            &Test {
                term: "http://example.com/1".to_owned(),
            },
            &[
                Token::Seq { len: Some(2) },
                Token::Map { len: None },
                Token::Str("id"),
                Token::Str("http://example.com/1"),
                Token::MapEnd,
                Token::Str("http://example.com/2"),
                Token::SeqEnd,
            ],
        );
    }
}
