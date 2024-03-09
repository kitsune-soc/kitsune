use super::DeserializerIntoDeserializer;
use core::{
    fmt::{self, Formatter},
    iter,
    marker::PhantomData,
};
use serde::de::{
    self,
    value::{
        BorrowedBytesDeserializer, BorrowedStrDeserializer, EnumAccessDeserializer,
        MapAccessDeserializer, SeqAccessDeserializer, SeqDeserializer,
    },
    Deserialize, DeserializeSeed, Deserializer, EnumAccess, MapAccess, SeqAccess,
};

/// Deserialises a JSON-LD set as a sequence.
pub struct Set<T> {
    seed: T,
}

struct Visitor<T>(T);

impl<'de, T> Set<PhantomData<T>>
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

impl<'de, T> Set<T>
where
    T: DeserializeSeed<'de>,
{
    pub fn with_seed(seed: T) -> Self {
        Self { seed }
    }
}

impl<'de, T> Default for Set<PhantomData<T>>
where
    T: Deserialize<'de>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, T> DeserializeSeed<'de> for Set<T>
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

macro_rules! forward_to_seq_deserializer {
    ($(fn $name:ident($T:ty);)*) => {$(
        fn $name<E>(self, v: $T) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            self.0.deserialize(serde::de::value::SeqDeserializer::new(core::iter::once(v)))
        }
    )*};
}

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = T::Value;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(super::EXPECTING_SET)
    }

    fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let iter = iter::once(DeserializerIntoDeserializer(BorrowedStrDeserializer::new(
            v,
        )));
        self.0.deserialize(SeqDeserializer::new(iter))
    }

    fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        let iter = iter::once(DeserializerIntoDeserializer(
            BorrowedBytesDeserializer::new(v),
        ));
        self.0.deserialize(SeqDeserializer::new(iter))
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.0.deserialize(SeqAccessDeserializer::new(seq))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_unit()
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let iter = iter::once(DeserializerIntoDeserializer(deserializer));
        self.0.deserialize(SeqDeserializer::new(iter))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        // TODO: Use `!` (`Infallible`) when it implements `IntoDeserializer`.
        let iter = iter::empty::<()>();
        self.0.deserialize(SeqDeserializer::new(iter))
    }

    fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let iter = iter::once(DeserializerIntoDeserializer(deserializer));
        self.0.deserialize(SeqDeserializer::new(iter))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let iter = iter::once(DeserializerIntoDeserializer(MapAccessDeserializer::new(
            map,
        )));
        self.0.deserialize(SeqDeserializer::new(iter))
    }

    fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error>
    where
        A: EnumAccess<'de>,
    {
        let iter = iter::once(DeserializerIntoDeserializer(EnumAccessDeserializer::new(
            data,
        )));
        self.0.deserialize(SeqDeserializer::new(iter))
    }

    forward_to_seq_deserializer! {
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
        fn visit_byte_buf(Vec<u8>);
    }
}

#[cfg(test)]
mod tests {
    use super::{super::into_deserializer, Set};

    #[test]
    fn single() {
        let data = 42;
        assert_eq!(Set::deserialize(into_deserializer(data)), Ok(vec![data]));
    }

    #[test]
    fn seq() {
        let data = vec![42, 21];
        assert_eq!(Set::deserialize(into_deserializer(data.clone())), Ok(data));
    }

    #[test]
    fn empty() {
        let data: Vec<u32> = Vec::new();
        assert_eq!(Set::deserialize(into_deserializer(data.clone())), Ok(data));
    }

    #[test]
    fn unit() {
        #[allow(clippy::let_unit_value)]
        let data = ();

        assert_eq!(
            Set::deserialize(into_deserializer(data)),
            Ok(Vec::<u32>::new())
        );
    }
}
