use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::{
    de::{
        self, value::SeqAccessDeserializer, DeserializeSeed, Deserializer, IgnoredAny, MapAccess,
        SeqAccess,
    },
    Deserialize,
};

/// Deserialises a single node identifier string or a set of node identifier strings.
pub struct Id<T> {
    seed: T,
}

struct Visitor<T>(T);

#[cfg_attr(not(test), allow(dead_code))]
impl<'de, T> Id<PhantomData<T>>
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

impl<'de, T> Id<T>
where
    T: DeserializeSeed<'de>,
{
    pub fn with_seed(seed: T) -> Self {
        Self { seed }
    }
}

impl<'de, T> Default for Id<PhantomData<T>>
where
    T: Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, T> DeserializeSeed<'de> for Id<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor(self.seed))
    }
}

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("a JSON-LD node")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "camelCase")]
        enum Key {
            #[serde(alias = "@id")]
            Id,
            #[serde(other)]
            Other,
        }

        while let Some(key) = map.next_key()? {
            match key {
                Key::Id => {
                    let value = map.next_value_seed(self.0)?;
                    while let Some((IgnoredAny, IgnoredAny)) = map.next_entry()? {}
                    return Ok(value);
                }
                Key::Other => {
                    let IgnoredAny = map.next_value()?;
                }
            }
        }

        Err(de::Error::missing_field("id"))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        struct SeqAccess<A>(A);

        impl<'de, A> de::SeqAccess<'de> for SeqAccess<A>
        where
            A: de::SeqAccess<'de>,
        {
            type Error = A::Error;

            fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: DeserializeSeed<'de>,
            {
                self.0.next_element_seed(Id::with_seed(seed))
            }

            fn size_hint(&self) -> Option<usize> {
                self.0.size_hint()
            }
        }

        self.0
            .deserialize(SeqAccessDeserializer::new(SeqAccess(seq)))
    }

    forward_to_into_deserializer! {
        fn visit_str(&str);
        fn visit_borrowed_str(&'de str);
        fn visit_string(String);
        fn visit_bytes(&[u8]);
        fn visit_borrowed_bytes(&'de [u8]);
        fn visit_byte_buf(Vec<u8>);
    }
}

#[cfg(test)]
mod tests {
    use super::{super::into_deserializer, Id};
    use core::marker::PhantomData;
    use serde::Deserialize;
    use serde_test::{assert_de_tokens, Token};
    use std::collections::HashMap;

    #[test]
    fn single() {
        let data = "http://example.com/".to_owned();
        assert_eq!(Id::deserialize(into_deserializer(&data[..])), Ok(data));
    }

    #[test]
    fn single_embedded() {
        let data: HashMap<_, _> = [("id", "http://example.com/")].into_iter().collect();
        assert_eq!(
            Id::deserialize(into_deserializer(data)),
            Ok("http://example.com/".to_owned())
        );
    }

    #[test]
    fn embedded_missing_id() {
        let data: HashMap<_, _> = [("foo", "http://example.com/")].into_iter().collect();
        assert!(Id::<PhantomData<String>>::deserialize(into_deserializer(data)).is_err());
    }

    #[test]
    fn seq() {
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(transparent)]
        struct Test {
            #[serde(deserialize_with = "Id::deserialize")]
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
