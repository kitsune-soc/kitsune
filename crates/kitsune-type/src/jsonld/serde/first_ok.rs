use super::CatchError;
use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::de::{
    self,
    value::{EnumAccessDeserializer, MapAccessDeserializer},
    Deserialize, DeserializeSeed, Deserializer, EnumAccess, IgnoredAny, IntoDeserializer,
    MapAccess, SeqAccess,
};

// XXX: Conceptually, we could decompose it into `First` and a helper type that filters successfully
// deserialised elements in a JSON-LD set. In practice, however, the latter type cannot be
// implemented (at least straightforwardly) because it would need to hook the
// `SeqAccess::next_element_seed` method, where we cannot clone the generic seed value like we're
// doing in `Visitor::visit_seq` below.

/// Deserialises a single element from a JSON-LD set.
///
/// It tries to deserialise each of the elements in the set and returns the first one successfully
/// deserialised.
///
/// The detection of recoverable errors is a "best effort" check and won't work for maps for
/// example, although it works for strings. It's suitable for tag-like fields like `"type"`.
pub struct FirstOk<T> {
    seed: T,
}

struct Visitor<T>(T);

impl<'de, T> FirstOk<PhantomData<T>>
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

impl<'de, T> FirstOk<T>
where
    T: DeserializeSeed<'de> + Clone,
{
    pub fn with_seed(seed: T) -> Self {
        Self { seed }
    }
}

impl<'de, T> DeserializeSeed<'de> for FirstOk<T>
where
    T: DeserializeSeed<'de> + Clone,
{
    type Value = T::Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor(self.seed))
    }
}

macro_rules! forward_to_into_deserializer {
    ($(fn $name:ident($T:ty);)*) => {$(
        fn $name<E>(self, v: $T) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            self.0
                .clone()
                .deserialize(serde::de::IntoDeserializer::into_deserializer(v))
                // No (deserialisable) element in the (single-value) set.
                // Interpret it as equivalent to `null` according to the JSON-LD data model.
                .or_else(|_: E| self.0.deserialize(serde::de::IntoDeserializer::into_deserializer(())))
        }
    )*};
}

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: DeserializeSeed<'de> + Clone,
{
    type Value = T::Value;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(super::EXPECTING_SET)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        loop {
            match seq.next_element_seed(CatchError::<_, A::Error>::new(self.0.clone()))? {
                Some(Ok(value)) => {
                    while let Some(IgnoredAny) = seq.next_element()? {}
                    return Ok(value);
                }
                Some(Err(_)) => {}
                None => return self.0.deserialize(().into_deserializer()),
            }
        }
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0
            .clone()
            .deserialize(de::value::BorrowedStrDeserializer::new(v))
            .or_else(|_: E| self.0.deserialize(().into_deserializer()))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0
            .clone()
            .deserialize(de::value::BorrowedBytesDeserializer::new(v))
            .or_else(|_: E| self.0.deserialize(().into_deserializer()))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0.deserialize(().into_deserializer())
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        FirstOk::with_seed(self.0).deserialize(deserializer)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.0.deserialize(().into_deserializer())
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        FirstOk::with_seed(self.0).deserialize(deserializer)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.0.deserialize(MapAccessDeserializer::new(map))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        self.0.deserialize(EnumAccessDeserializer::new(data))
    }

    forward_to_into_deserializer! {
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
mod tests {
    use serde::Deserialize;

    use super::super::into_deserializer;
    use super::FirstOk;

    #[test]
    fn simple() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Test {
            A,
        }
        let data = "A";
        assert_eq!(FirstOk::deserialize(into_deserializer(data)), Ok(Test::A));
    }

    #[test]
    fn simple_fail() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Test {
            A,
        }

        let data = "B";
        assert_eq!(
            FirstOk::deserialize(into_deserializer(data)),
            Ok(None::<Test>)
        );
    }

    #[test]
    fn seq() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Test {
            A,
            B,
        }

        let data = vec!["C", "B", "A"];
        assert_eq!(FirstOk::deserialize(into_deserializer(data)), Ok(Test::B));
    }

    #[test]
    fn seq_fail() {
        #[derive(Debug, Deserialize, PartialEq)]
        enum Test {
            A,
            B,
        }

        let data = vec!["C", "D"];
        assert_eq!(
            FirstOk::deserialize(into_deserializer(data)),
            Ok(None::<Test>)
        );
    }
}
