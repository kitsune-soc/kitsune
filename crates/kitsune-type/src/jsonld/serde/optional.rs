use core::fmt::{self, Formatter};
use serde::de::{self, DeserializeSeed, Deserializer};

/// Deserialises an `Option<T::Value>` value.
///
/// Workaround until Serde introduces a native mechanism for applying the
/// `#[serde(deserialize_with)]` attribute to the type inside an `Option<_>`.
///
/// cf. <https://github.com/serde-rs/serde/issues/723>.
pub struct Optional<T> {
    seed: T,
}

struct Visitor<T>(T);

impl<'de, T> Optional<T>
where
    T: DeserializeSeed<'de> + Default,
{
    pub fn new() -> Self {
        Self::with_seed(T::default())
    }

    pub fn deserialize<D>(deserializer: D) -> Result<Option<T::Value>, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new().deserialize(deserializer)
    }
}

impl<'de, T> Optional<T>
where
    T: DeserializeSeed<'de>,
{
    pub fn with_seed(seed: T) -> Self {
        Self { seed }
    }
}

impl<'de, T> DeserializeSeed<'de> for Optional<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = Option<T::Value>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_option(Visitor(self.seed))
    }
}

impl<'de, T> de::Visitor<'de> for Visitor<T>
where
    T: DeserializeSeed<'de>,
{
    type Value = Option<T::Value>;

    fn expecting(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("option")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        self.0.deserialize(deserializer).map(Some)
    }
}
