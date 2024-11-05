//! Serde helpers to translate JSON-LD data structures.
//!
//! ## JSON-LD `@set`
//!
//! When a JSON-LD term's `@container` is unspecified or is set to `@set`, JSON entry values in the
//! following groups are semantically equivalent, respectively:
//!
//! - A non-array value (`"value"`) and a single-value array of the same value (`["value"]`)
//! - An empty array (`[]`), `null` and an absent entry
//!
//!
//! The following helpers deserialise a single value or `null` from a set:
//!
//! - [`First`]
//! - [`FirstOk`]
//!
//! ## JSON-LD `@id`
//!
//! When a JSON-LD term's `@type` is set to `@id`, a JSON entry value of a single (IRI) string
//! (`"http://example.com/"`) is a shorthand for a Linked Data node identified by that string
//! (`{"@id": "http://example.com/"}`.
//!
//! The following helpers deserialise the node identifier string(s):
//!
//! - [`Id`]

macro_rules! forward_to_into_deserializer {
    (
        fn visit_borrowed_str($T:ty);
        $($rest:tt)*
    ) => {
        fn visit_borrowed_str<E>(self, v: $T) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            T::deserialize(serde::de::value::BorrowedStrDeserializer::new(v))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_borrowed_bytes($T:ty);
        $($rest:tt)*
    ) => {
        fn visit_borrowed_bytes<E>(self, v: $T) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            T::deserialize(serde::de::value::BorrowedBytesDeserializer::new(v))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_none();
        $($rest:tt)*
    ) => {
        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            T::deserialize(serde::de::IntoDeserializer::into_deserializer(()))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_some();
        $($rest:tt)*
    ) => {
        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            T::deserialize(deserializer)
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_unit();
        $($rest:tt)*
    ) => {
        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            T::deserialize(serde::de::IntoDeserializer::into_deserializer(()))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_newtype_struct();
        $($rest:tt)*
    ) => {
        fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            T::deserialize(deserializer)
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_seq();
        $($rest:tt)*
    ) => {
        fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            T::deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_map();
        $($rest:tt)*
    ) => {
        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            T::deserialize(serde::de::value::MapAccessDeserializer::new(map))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn visit_enum();
        $($rest:tt)*
    ) => {
        fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::EnumAccess<'de>,
        {
            T::deserialize(serde::de::value::EnumAccessDeserializer::new(data))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    (
        fn $name:ident($T:ty);
        $($rest:tt)*
    ) => {
        fn $name<E>(self, v: $T) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            T::deserialize(serde::de::IntoDeserializer::into_deserializer(v))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    () => {};
}

mod first;
mod first_ok;
mod id;

pub use self::{first::First, first_ok::FirstOk, id::Id};

use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::{
    de::{
        self,
        value::{EnumAccessDeserializer, MapAccessDeserializer, SeqAccessDeserializer},
        Deserializer, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, Visitor,
    },
    Deserialize,
};
use serde_with::DeserializeAs;

const EXPECTING_SET: &str = "a JSON-LD set";

/// A `DeserializeSeed` that catches a recoverable error and returns it as a successful value.
struct CatchError<T, E> {
    target: PhantomData<T>,
    marker: PhantomData<fn() -> E>,
}

impl<'de, T, E> DeserializeAs<'de, Result<T, E>> for CatchError<T, E>
where
    T: Deserialize<'de>,
    E: de::Error,
{
    fn deserialize_as<D>(deserializer: D) -> Result<Result<T, E>, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(CatchError {
            target: PhantomData,
            marker: PhantomData,
        })
    }
}

macro_rules! catch_error_forward_to_into_deserializer {
    ($(fn $name:ident($T:ty);)*) => {$(
        fn $name<E2>(self, v: $T) -> Result<Self::Value, E2> {
            // We can tell that the error isn't fatal to the deserialiser because it's originated
            // from the already deserialised value `$t` rather than the deserialiser.
            Ok(T::deserialize(serde::de::IntoDeserializer::into_deserializer(v)))
        }
    )*};
}

impl<'de, T, E> Visitor<'de> for CatchError<T, E>
where
    T: Deserialize<'de>,
    E: de::Error,
{
    type Value = Result<T, E>;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("a value")
    }

    fn visit_borrowed_str<E2>(self, v: &'de str) -> Result<Self::Value, E2>
    where
        E: de::Error,
    {
        Ok(T::deserialize(de::value::BorrowedStrDeserializer::new(v)))
    }

    fn visit_borrowed_bytes<E2>(self, v: &'de [u8]) -> Result<Self::Value, E2>
    where
        E: de::Error,
    {
        Ok(T::deserialize(de::value::BorrowedBytesDeserializer::new(v)))
    }

    fn visit_none<E2>(self) -> Result<Self::Value, E2> {
        Ok(T::deserialize(().into_deserializer()))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize_as(deserializer)
    }

    fn visit_unit<E2>(self) -> Result<Self::Value, E2> {
        Ok(T::deserialize(().into_deserializer()))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize_as(deserializer)
    }

    // XXX: The following methods cannot determine whether an error is recoverable. While we might
    // be able to implement them _right_ way by hooking into the `*Access` trait implementations and
    // recursively applying `CatchError` in the `*_seed` method calls, that wouldn't be worth the
    // effort since we're currently not using `FirstOk` (the only user of `CatchError` as of now)
    // for these types, and as for `visit_seq`, JSON-LD doesn't support nested sets anyway.
    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Self::deserialize_as(SeqAccessDeserializer::new(seq))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        Self::deserialize_as(MapAccessDeserializer::new(map))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        Self::deserialize_as(EnumAccessDeserializer::new(data))
    }

    catch_error_forward_to_into_deserializer! {
        fn visit_bool(bool);
        fn visit_i8(i8);
        fn visit_i16(i16);
        fn visit_i32(i32);
        fn visit_i64(i64);
        fn visit_i128(i128);
        fn visit_u8(u8);
        fn visit_u16(u16);
        fn visit_u32(u32);
        fn visit_u64(u64);
        fn visit_u128(u128);
        fn visit_f32(f32);
        fn visit_f64(f64);
        fn visit_char(char);
        fn visit_str(&str);
        fn visit_string(String);
        fn visit_bytes(&[u8]);
        fn visit_byte_buf(Vec<u8>);
    }
}

#[cfg(test)]
fn into_deserializer<'de, T>(value: T) -> T::Deserializer
where
    T: serde::de::IntoDeserializer<'de, serde::de::value::Error>,
{
    serde::de::IntoDeserializer::into_deserializer(value)
}

#[cfg(test)]
mod tests {
    use super::{First, FirstOk, Id};
    use serde::Deserialize;
    use serde_with::{serde_as, OneOrMany};

    /// Checks that the types work for some random real-world-ish use cases.
    #[test]
    fn integrate() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Type {
            Note,
        }

        #[serde_as]
        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            id: String,
            #[serde_as(as = "FirstOk")]
            r#type: Type,
            #[serde_as(as = "First<Id>")]
            attributed_to: String,
            #[serde(default)]
            #[serde_as(as = "Option<First>")]
            summary: Option<String>,
            #[serde(default)]
            #[serde_as(as = "Option<First>")]
            content: Option<String>,
            #[serde(default)]
            #[serde_as(as = "OneOrMany<Id>")]
            to: Vec<String>,
            #[serde(default)]
            #[serde_as(as = "OneOrMany<Id>")]
            cc: Vec<String>,
        }

        let object = br#"
        {
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": "https://example.com/notes/1",
            "type": "Note",
            "attributedTo": "https://example.com/actors/1",
            "summary": "An ordinary Note",
            "content": "Hello, world!",
            "to": ["https://example.com/actors/2"],
            "cc": ["https://www.w3.org/ns/activitystreams#Public"]
        }
        "#;
        let expected = Object {
            id: "https://example.com/notes/1".to_owned(),
            r#type: Type::Note,
            attributed_to: "https://example.com/actors/1".to_owned(),
            summary: Some("An ordinary Note".to_owned()),
            content: Some("Hello, world!".to_owned()),
            to: vec!["https://example.com/actors/2".to_owned()],
            cc: vec!["https://www.w3.org/ns/activitystreams#Public".to_owned()],
        };
        assert_eq!(sonic_rs::from_slice::<Object>(object).unwrap(), expected);

        let object = br#"
        {
            "@context": "https://www.w3.org/ns/activitystreams",
            "id": "https://example.com/notes/1",
            "type": ["http://schema.org/CreativeWork", "Note"],
            "attributedTo": [
                {
                    "id": "https://example.com/actors/1",
                    "type": "Person"
                },
                "https://example.com/actors/2"
            ],
            "summary": "A quirky Note",
            "to": "https://example.com/actors/3"
        }
        "#;
        let expected = Object {
            id: "https://example.com/notes/1".to_owned(),
            // Multiple `type`s including unknown ones:
            r#type: Type::Note,
            // Multiple `attributedTo`s and an embedded node:
            attributed_to: "https://example.com/actors/1".to_owned(),
            summary: Some("A quirky Note".to_owned()),
            // Absent `Option` field:
            content: None,
            // Single-value set:
            to: vec!["https://example.com/actors/3".to_owned()],
            // Absent `serde(default)` field:
            cc: vec![],
        };
        assert_eq!(sonic_rs::from_slice::<Object>(object).unwrap(), expected);
    }
}
