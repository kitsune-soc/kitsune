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
//! The following helpers in the module deserialise a set as a sequence:
//!
//! - [`Set`]
//! - [`IdSet`]
//!
//! The following helpers deserialise a single value or `null` from a set:
//!
//! - [`First`]
//! - [`FirstId`]
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
//! - [`FirstId`]
//! - [`IdSet`]

macro_rules! forward_to_into_deserializer {
    (
        fn visit_borrowed_str($T:ty);
        $($rest:tt)*
    ) => {
        fn visit_borrowed_str<E>(self, v: $T) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.0.deserialize(serde::de::value::BorrowedStrDeserializer::new(v))
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
            self.0.deserialize(serde::de::value::BorrowedBytesDeserializer::new(v))
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
            self.0.deserialize(serde::de::IntoDeserializer::into_deserializer(()))
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
            self.0.deserialize(deserializer)
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
            self.0.deserialize(serde::de::IntoDeserializer::into_deserializer(()))
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
            self.0.deserialize(deserializer)
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
            self.0.deserialize(serde::de::value::SeqAccessDeserializer::new(seq))
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
            self.0.deserialize(serde::de::value::MapAccessDeserializer::new(map))
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
            self.0.deserialize(serde::de::value::EnumAccessDeserializer::new(data))
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
            self.0.deserialize(serde::de::IntoDeserializer::into_deserializer(v))
        }
        forward_to_into_deserializer! { $($rest)* }
    };
    () => {};
}

mod first;
mod first_id;
mod first_ok;
mod id;
mod id_set;
mod optional;
mod set;

pub use self::first::First;
pub use self::first_id::FirstId;
pub use self::first_ok::FirstOk;
pub use self::id::Id;
pub use self::id_set::IdSet;
pub use self::optional::Optional;
pub use self::set::Set;

use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::de::{
    self,
    value::{EnumAccessDeserializer, MapAccessDeserializer, SeqAccessDeserializer},
    DeserializeSeed, Deserializer, EnumAccess, IntoDeserializer, MapAccess, SeqAccess, Visitor,
};

const EXPECTING_SET: &str = "a JSON-LD set";

/// A wrapper to implement `IntoDeserializer` for an `impl Deserializer`, because Serde somehow
/// doesn't provide a blanket impl.
struct DeserializerIntoDeserializer<D>(D);

/// A `DeserializeSeed` that catches a recoverable error and returns it as a successful value.
struct CatchError<T, E> {
    seed: T,
    marker: PhantomData<fn() -> E>,
}

struct OptionSeed<T>(Option<T>);

impl<'de, D> IntoDeserializer<'de, D::Error> for DeserializerIntoDeserializer<D>
where
    D: Deserializer<'de>,
{
    type Deserializer = D;

    fn into_deserializer(self) -> Self::Deserializer {
        self.0
    }
}

impl<'de, T> DeserializeSeed<'de> for &mut OptionSeed<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = Option<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        if let Some(seed) = self.0.take() {
            seed.deserialize(deserializer).map(Some)
        } else {
            Ok(None)
        }
    }
}

impl<'de, T, E> CatchError<T, E>
where
    T: DeserializeSeed<'de>,
    E: de::Error,
{
    pub fn new(seed: T) -> Self {
        Self {
            seed,
            marker: PhantomData,
        }
    }
}

impl<'de, T, E> DeserializeSeed<'de> for CatchError<T, E>
where
    T: DeserializeSeed<'de>,
    E: de::Error,
{
    type Value = Result<T::Value, E>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(self)
    }
}

macro_rules! catch_error_forward_to_into_deserializer {
    ($(fn $name:ident($T:ty);)*) => {$(
        fn $name<E2>(self, v: $T) -> Result<Self::Value, E2> {
            // We can tell that the error isn't fatal to the deserialiser because it's originated
            // from the already deserialised value `$t` rather than the deserialiser.
            Ok(self.seed.deserialize(serde::de::IntoDeserializer::into_deserializer(v)))
        }
    )*};
}

impl<'de, T, E> Visitor<'de> for CatchError<T, E>
where
    T: DeserializeSeed<'de>,
    E: de::Error,
{
    type Value = Result<T::Value, E>;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("a value")
    }

    fn visit_borrowed_str<E2>(self, v: &'de str) -> Result<Self::Value, E2>
    where
        E: de::Error,
    {
        Ok(self
            .seed
            .deserialize(de::value::BorrowedStrDeserializer::new(v)))
    }

    fn visit_borrowed_bytes<E2>(self, v: &'de [u8]) -> Result<Self::Value, E2>
    where
        E: de::Error,
    {
        Ok(self
            .seed
            .deserialize(de::value::BorrowedBytesDeserializer::new(v)))
    }

    fn visit_none<E2>(self) -> Result<Self::Value, E2> {
        Ok(self.seed.deserialize(().into_deserializer()))
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.deserialize(deserializer)
    }

    fn visit_unit<E2>(self) -> Result<Self::Value, E2> {
        Ok(self.seed.deserialize(().into_deserializer()))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.deserialize(deserializer)
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
        self.seed
            .deserialize(SeqAccessDeserializer::new(seq))
            .map(Ok)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.seed
            .deserialize(MapAccessDeserializer::new(map))
            .map(Ok)
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        self.seed
            .deserialize(EnumAccessDeserializer::new(data))
            .map(Ok)
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
    use super::{First, FirstId, FirstOk, IdSet, Optional};
    use serde::Deserialize;

    /// Checks that the types work for some random real-world-ish use cases.
    #[test]
    fn integrate() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Type {
            Note,
        }

        #[derive(Debug, Deserialize, PartialEq)]
        #[serde(rename_all = "camelCase")]
        struct Object {
            id: String,
            #[serde(deserialize_with = "FirstOk::deserialize")]
            r#type: Type,
            #[serde(deserialize_with = "FirstId::deserialize")]
            attributed_to: String,
            #[serde(default)]
            #[serde(deserialize_with = "Optional::<First<_>>::deserialize")]
            summary: Option<String>,
            #[serde(default)]
            #[serde(deserialize_with = "Optional::<First<_>>::deserialize")]
            content: Option<String>,
            #[serde(default)]
            #[serde(deserialize_with = "IdSet::deserialize")]
            to: Vec<String>,
            #[serde(default)]
            #[serde(deserialize_with = "IdSet::deserialize")]
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
        assert_eq!(simd_json::from_slice(&mut object.to_owned()), Ok(expected));

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
        assert_eq!(simd_json::from_slice(&mut object.to_owned()), Ok(expected));
    }
}
