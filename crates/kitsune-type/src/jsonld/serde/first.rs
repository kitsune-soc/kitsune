use super::OptionSeed;
use core::{
    fmt::{self, Formatter},
    marker::PhantomData,
};
use serde::de::{
    self, Deserialize, DeserializeSeed, Deserializer, IgnoredAny, IntoDeserializer, SeqAccess,
};

/// Deserialises the first element of a JSON-LD set.
pub struct First<T> {
    seed: T,
}

struct Visitor<T>(T);

impl<'de, T> First<PhantomData<T>>
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

impl<'de, T> First<T>
where
    T: DeserializeSeed<'de>,
{
    pub fn with_seed(seed: T) -> Self {
        Self { seed }
    }
}

impl<'de, T> DeserializeSeed<'de> for First<T>
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
        f.write_str(super::EXPECTING_SET)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut seed = OptionSeed(Some(self.0));
        let value = if let Some(value) = seq.next_element_seed(&mut seed)? {
            // Unwrapping is fine here because the first call to `OptionSeed::deserialize` always
            // returns a `Some` and `next_element_seed` can only call it at most once because its
            // signature takes the seed by value.
            let value = value.unwrap();
            while let Some(IgnoredAny) = seq.next_element()? {}
            value
        } else if let Some(seed) = seed.0 {
            seed.deserialize(().into_deserializer())?
        } else {
            // Weirdly, the `SeqAccess` has consumed the seed yet it didn't return a value.
            return Err(de::Error::invalid_length(0, &super::EXPECTING_SET));
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
    use core::marker::PhantomData;

    #[test]
    fn single() {
        let data = 42;
        assert_eq!(First::deserialize(into_deserializer(data)), Ok(data));
    }

    #[test]
    fn seq() {
        let data = vec![42, 21];
        assert_eq!(First::deserialize(into_deserializer(data)), Ok(42));
    }

    #[test]
    fn empty() {
        let data: Vec<u32> = Vec::new();
        assert_eq!(
            First::<PhantomData<Option<u32>>>::deserialize(into_deserializer(data)),
            Ok(None)
        );
    }
}
