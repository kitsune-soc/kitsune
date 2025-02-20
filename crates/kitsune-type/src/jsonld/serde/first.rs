use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::{
    de::{self, Deserialize, Deserializer, IgnoredAny, IntoDeserializer, SeqAccess},
    Serialize,
};
use serde_with::{de::DeserializeAsWrap, DeserializeAs, SerializeAs};

/// Deserialises the first element of a JSON-LD set.
#[allow(dead_code)] // Used inside `serde_as` macro.
pub struct First<U = serde_with::Same>(PhantomData<U>);

impl<'de, T, U> DeserializeAs<'de, T> for First<U>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(Visitor(PhantomData::<T>, PhantomData::<U>))
    }
}

impl<T, U> SerializeAs<T> for First<U>
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

struct Visitor<T, U>(PhantomData<T>, PhantomData<U>);

impl<'de, T, U> de::Visitor<'de> for Visitor<T, U>
where
    T: Deserialize<'de>,
    U: DeserializeAs<'de, T>,
{
    type Value = T;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(super::EXPECTING_SET)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let value = match seq.next_element::<DeserializeAsWrap<_, U>>()? {
            Some(value) => {
                while let Some(IgnoredAny) = seq.next_element()? {}
                value.into_inner()
            }
            _ => U::deserialize_as(().into_deserializer())?,
        };

        Ok(value)
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
        fn visit_borrowed_str(&'de str);
        fn visit_string(String);
        fn visit_bytes(&[u8]);
        fn visit_borrowed_bytes(&'de [u8]);
        fn visit_byte_buf(Vec<u8>);
        fn visit_none();
        fn visit_some();
        fn visit_unit();
        fn visit_newtype_struct();
        fn visit_map();
        fn visit_enum();
    }
}

#[cfg(test)]
mod tests {
    use super::{super::into_deserializer, First};
    use serde_with::{DeserializeAs, Same};

    #[test]
    fn single() {
        let data = 42;
        assert_eq!(
            First::<Same>::deserialize_as(into_deserializer(data)),
            Ok(data)
        );
    }

    #[test]
    fn seq() {
        let data = vec![42, 21];
        assert_eq!(
            First::<Same>::deserialize_as(into_deserializer(data)),
            Ok(42)
        );
    }

    #[test]
    fn empty() {
        let data: Vec<u32> = Vec::new();
        let first: Result<Option<u32>, _> = First::<Same>::deserialize_as(into_deserializer(data));
        assert_eq!(first, Ok(None));
    }
}
