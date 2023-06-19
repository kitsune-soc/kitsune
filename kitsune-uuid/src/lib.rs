#![forbid(rust_2018_idioms)]
#![warn(clippy::all, clippy::pedantic)]

use diesel::{AsExpression, FromSqlRow};
use serde::{
    de::{self, Error as _},
    Deserialize, Serialize,
};
use std::{
    fmt,
    ops::{Deref, DerefMut},
    str::{self, FromStr},
};
use uuid_simd::{format_hyphenated, AsciiCase, Out, UuidExt};

macro_rules! next_element {
    ($seq:ident, $self:ident) => {
        match $seq.next_element()? {
            Some(e) => e,
            None => return Err(A::Error::invalid_length(16, &$self)),
        }
    };
}

#[derive(AsExpression, Clone, Copy, Debug, FromSqlRow, PartialEq, Eq, PartialOrd, Ord)]
#[diesel(sql_type = diesel::sql_types::Uuid)]
#[repr(transparent)]
pub struct Uuid(pub uuid::Uuid);

impl AsRef<uuid::Uuid> for Uuid {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl Deref for Uuid {
    type Target = uuid::Uuid;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Uuid {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn de_error<E: de::Error>(e: impl fmt::Display) -> E {
            E::custom(format_args!("UUID parsing failed: {e}"))
        }

        if deserializer.is_human_readable() {
            struct UuidVisitor;

            impl<'vi> de::Visitor<'vi> for UuidVisitor {
                type Value = Uuid;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(formatter, "a UUID string")
                }

                fn visit_str<E: de::Error>(self, value: &str) -> Result<Uuid, E> {
                    value.parse().map_err(de_error)
                }

                fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Uuid, E> {
                    uuid::Uuid::from_slice(value).map(Uuid).map_err(de_error)
                }

                fn visit_seq<A>(self, mut seq: A) -> Result<Uuid, A::Error>
                where
                    A: de::SeqAccess<'vi>,
                {
                    let bytes = [
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                        next_element!(seq, self),
                    ];

                    Ok(Uuid(uuid::Uuid::from_bytes(bytes)))
                }
            }

            deserializer.deserialize_str(UuidVisitor)
        } else {
            struct UuidBytesVisitor;

            impl<'vi> de::Visitor<'vi> for UuidBytesVisitor {
                type Value = Uuid;

                fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(formatter, "bytes")
                }

                fn visit_bytes<E: de::Error>(self, value: &[u8]) -> Result<Uuid, E> {
                    uuid::Uuid::from_slice(value).map(Uuid).map_err(de_error)
                }
            }

            deserializer.deserialize_bytes(UuidBytesVisitor)
        }
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dst = [0; 36];
        let display = unsafe {
            str::from_utf8_unchecked(format_hyphenated(
                self.0.as_bytes(),
                Out::from_mut(&mut dst),
                AsciiCase::Lower,
            ))
        };

        write!(f, "{display}")
    }
}

impl From<uuid::Uuid> for Uuid {
    fn from(value: uuid::Uuid) -> Self {
        Self(value)
    }
}

impl From<Uuid> for uuid::Uuid {
    fn from(value: Uuid) -> Self {
        value.0
    }
}

impl FromStr for Uuid {
    type Err = uuid_simd::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(uuid::Uuid::parse(s)?))
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut dst = [0; 36];
        serializer.serialize_str(unsafe {
            str::from_utf8_unchecked(format_hyphenated(
                self.0.as_bytes(),
                Out::from_mut(&mut dst),
                AsciiCase::Lower,
            ))
        })
    }
}

#[cfg(test)]
mod test {
    use crate::Uuid;
    use std::str::FromStr;

    const UUID_1: &str = "38058daf-b2cd-4832-902a-83583ac07e28";

    #[test]
    fn parse_1() {
        let uuid = Uuid::from_str(UUID_1).unwrap();
        assert_eq!(UUID_1, uuid.to_string());
    }
}
