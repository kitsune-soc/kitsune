use std::{
    fmt::{self, Debug},
    ops::{Deref, DerefMut},
    str::{self, FromStr},
};
use thiserror::Error;
use uuid_simd::{format_hyphenated, AsciiCase, Out, UuidExt};

pub use uuid;

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Simd(#[from] uuid_simd::Error),

    #[error(transparent)]
    Uuid(#[from] uuid::Error),
}

#[cfg_attr(feature = "diesel", derive(diesel::AsExpression, diesel::FromSqlRow))]
#[cfg_attr(feature = "diesel", diesel(sql_type = diesel::sql_types::Uuid))]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Uuid(pub uuid::Uuid);

impl Uuid {
    fn as_ascii_bytes(&self) -> [u8; 36] {
        let mut dst = [0; 36];
        let _ = format_hyphenated(self.0.as_bytes(), Out::from_mut(&mut dst), AsciiCase::Lower);
        dst
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self, Error> {
        uuid::Uuid::from_slice(bytes).map(Self).map_err(Error::from)
    }

    #[must_use]
    pub fn max() -> Self {
        Self(uuid::Uuid::max())
    }

    #[must_use]
    pub fn new_v7(ts: uuid::Timestamp) -> Self {
        Self(uuid::Uuid::new_v7(ts))
    }

    #[must_use]
    pub fn nil() -> Self {
        Self(uuid::Uuid::nil())
    }

    #[must_use]
    pub fn now_v7() -> Self {
        Self(uuid::Uuid::now_v7())
    }
}

impl AsRef<uuid::Uuid> for Uuid {
    fn as_ref(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl Debug for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
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

impl fmt::Display for Uuid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.as_ascii_bytes();

        #[allow(unsafe_code)]
        // Safety: The `uuid-simd` library provides a buffer of correctly encoded UTF-8 bytes
        let display = unsafe { str::from_utf8_unchecked(&bytes) };

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
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(uuid::Uuid::parse(s)?))
    }
}

#[cfg(feature = "async-graphql")]
mod async_graphql_impl {
    use super::Uuid;
    use async_graphql::{
        connection::CursorType, InputValueError, InputValueResult, Scalar, ScalarType, Value,
    };
    use std::str::FromStr;

    impl CursorType for Uuid {
        type Error = crate::Error;

        fn decode_cursor(s: &str) -> Result<Self, Self::Error> {
            s.parse()
        }

        fn encode_cursor(&self) -> String {
            self.to_string()
        }
    }

    /// A UUID is a unique 128-bit number, stored as 16 octets. UUIDs are parsed as
    /// Strings within GraphQL. UUIDs are used to assign unique identifiers to
    /// entities without requiring a central allocating authority.
    ///
    /// # References
    ///
    /// * [Wikipedia: Universally Unique Identifier](http://en.wikipedia.org/wiki/Universally_unique_identifier)
    /// * [RFC4122: A Universally Unique Identifier (UUID) URN Namespace](http://tools.ietf.org/html/rfc4122)
    #[Scalar(name = "UUID", specified_by_url = "http://tools.ietf.org/html/rfc4122")]
    impl ScalarType for Uuid {
        fn parse(value: Value) -> InputValueResult<Self> {
            match value {
                Value::String(s) => Ok(Uuid::from_str(&s)?),
                _ => Err(InputValueError::expected_type(value)),
            }
        }

        fn to_value(&self) -> Value {
            Value::String(self.to_string())
        }
    }
}

#[cfg(feature = "diesel")]
mod diesel_impl {
    use crate::Uuid;
    use diesel::{backend::Backend, deserialize::FromSql, pg::Pg, serialize::ToSql, sql_types};

    impl FromSql<sql_types::Uuid, Pg> for Uuid {
        fn from_sql(bytes: <Pg as Backend>::RawValue<'_>) -> diesel::deserialize::Result<Self> {
            <uuid::Uuid as FromSql<sql_types::Uuid, Pg>>::from_sql(bytes).map(Self)
        }
    }

    impl ToSql<sql_types::Uuid, Pg> for Uuid {
        fn to_sql<'b>(
            &'b self,
            out: &mut diesel::serialize::Output<'b, '_, Pg>,
        ) -> diesel::serialize::Result {
            <uuid::Uuid as ToSql<sql_types::Uuid, Pg>>::to_sql(self, out)
        }
    }
}

#[cfg(feature = "redis")]
impl redis::ToRedisArgs for Uuid {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        let bytes = self.as_ascii_bytes();
        redis::ToRedisArgs::write_redis_args(&bytes.as_slice(), out);
    }
}

#[cfg(feature = "serde")]
mod serde_impl {
    use crate::Uuid;
    use serde::{
        de::{self, Error as _},
        Deserialize, Serialize,
    };
    use std::{fmt, str};

    macro_rules! next_element {
        ($seq:ident, $self:ident) => {
            match $seq.next_element()? {
                Some(e) => e,
                None => return Err(A::Error::invalid_length(16, &$self)),
            }
        };
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

    impl Serialize for Uuid {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let bytes = self.as_ascii_bytes();

            #[allow(unsafe_code)]
            // Safety: The `uuid-simd` library provides a buffer of correctly encoded UTF-8 bytes
            serializer.serialize_str(unsafe { str::from_utf8_unchecked(&bytes) })
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Uuid;
    use serde_test::Token;
    use std::str::FromStr;

    const UUID_1: &str = "38058daf-b2cd-4832-902a-83583ac07e28";
    const UUID_1_BYTES: [u8; 16] = [
        0x38, 0x05, 0x8d, 0xaf, 0xb2, 0xcd, 0x48, 0x32, 0x90, 0x2a, 0x83, 0x58, 0x3a, 0xc0, 0x7e,
        0x28,
    ];

    #[test]
    fn parse_1() {
        let uuid = Uuid::from_str(UUID_1).unwrap();
        assert_eq!(UUID_1, uuid.to_string());
    }

    #[test]
    #[cfg(feature = "serde")]
    fn deserialize_str() {
        use serde_test::Configure;

        let uuid = Uuid::from_str(UUID_1).unwrap().readable();
        serde_test::assert_de_tokens(&uuid, &[Token::Str(UUID_1)]);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn deserialize_bytes() {
        use serde_test::Configure;

        let uuid = Uuid::from_slice(&UUID_1_BYTES).unwrap();
        serde_test::assert_de_tokens(&uuid.compact(), &[Token::Bytes(&UUID_1_BYTES)]);
        serde_test::assert_de_tokens(&uuid.readable(), &[Token::Bytes(&UUID_1_BYTES)]);
    }

    #[test]
    #[cfg(feature = "serde")]
    fn deserialize_byte_array() {
        use serde_test::{Compact, Configure};

        let uuid = Uuid::from_slice(&UUID_1_BYTES).unwrap();
        serde_test::assert_de_tokens(
            &uuid.readable(),
            &[
                Token::Seq { len: Some(16) },
                Token::U8(UUID_1_BYTES[0]),
                Token::U8(UUID_1_BYTES[1]),
                Token::U8(UUID_1_BYTES[2]),
                Token::U8(UUID_1_BYTES[3]),
                Token::U8(UUID_1_BYTES[4]),
                Token::U8(UUID_1_BYTES[5]),
                Token::U8(UUID_1_BYTES[6]),
                Token::U8(UUID_1_BYTES[7]),
                Token::U8(UUID_1_BYTES[8]),
                Token::U8(UUID_1_BYTES[9]),
                Token::U8(UUID_1_BYTES[10]),
                Token::U8(UUID_1_BYTES[11]),
                Token::U8(UUID_1_BYTES[12]),
                Token::U8(UUID_1_BYTES[13]),
                Token::U8(UUID_1_BYTES[14]),
                Token::U8(UUID_1_BYTES[15]),
                Token::SeqEnd,
            ],
        );

        serde_test::assert_de_tokens_error::<Compact<Uuid>>(
            &[Token::Seq { len: Some(16) }],
            "invalid type: sequence, expected bytes",
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn serialize_uuid() {
        let uuid = Uuid::from_slice(&UUID_1_BYTES).unwrap();
        serde_test::assert_ser_tokens(&uuid, &[Token::Str(UUID_1)]);
    }
}
