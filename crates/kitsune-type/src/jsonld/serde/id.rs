use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::{
    de::{
        self, value::SeqAccessDeserializer, DeserializeSeed, Deserializer, IgnoredAny, MapAccess,
        SeqAccess,
    },
    Deserialize, Serialize,
};
use serde_with::{de::DeserializeAsWrap, DeserializeAs, SerializeAs};

/// Deserialises a single node identifier string or a set of node identifier strings.
#[allow(dead_code)] // Used inside `serde_as` macro.
pub struct Id;

impl<'de, T> DeserializeAs<'de, T> for Id
where
    T: Deserialize<'de>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor(PhantomData::<T>))
    }
}

impl<T> SerializeAs<T> for Id
where
    T: Serialize,
{
    fn serialize_as<S>(value: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        value.serialize(serializer)
    }
}

struct Visitor<T>(PhantomData<T>);

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: Deserialize<'de>,
{
    type Value = T;

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
                    let value = map.next_value()?;
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

            fn next_element<T>(&mut self) -> Result<Option<T>, Self::Error>
            where
                T: Deserialize<'de>,
            {
                let value = self.0.next_element::<DeserializeAsWrap<_, Id>>()?;
                Ok(value.map(DeserializeAsWrap::into_inner))
            }

            fn next_element_seed<T>(&mut self, _seed: T) -> Result<Option<T::Value>, Self::Error>
            where
                T: DeserializeSeed<'de>,
            {
                unimplemented!();
            }

            fn size_hint(&self) -> Option<usize> {
                self.0.size_hint()
            }
        }

        T::deserialize(SeqAccessDeserializer::new(SeqAccess(seq)))
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
    use serde::Deserialize;
    use serde_test::{assert_de_tokens, Token};
    use serde_with::{serde_as, DeserializeAs};
    use std::collections::HashMap;

    #[test]
    fn single() {
        let data = "http://example.com/";
        assert_eq!(
            Id::deserialize_as(into_deserializer(data)),
            Ok(data.to_owned())
        );
    }

    #[test]
    fn single_embedded() {
        let data: HashMap<_, _> = [("id", "http://example.com/")].into_iter().collect();
        assert_eq!(
            Id::deserialize_as(into_deserializer(data)),
            Ok("http://example.com/".to_owned())
        );
    }

    #[test]
    fn embedded_missing_id() {
        let data: HashMap<_, _> = [("foo", "http://example.com/")].into_iter().collect();
        let result: Result<String, _> = Id::deserialize_as(into_deserializer(data));
        assert!(result.is_err());
    }

    #[test]
    fn seq() {
        #[serde_as]
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(transparent)]
        struct Test {
            #[serde_as(as = "Id")]
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
