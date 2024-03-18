#![cfg(feature = "serde")]

use serde_test::{Compact, Configure, Token};
use speedy_uuid::Uuid;
use std::str::FromStr;

const UUID: &str = "38058daf-b2cd-4832-902a-83583ac07e28";
const UUID_BYTES: [u8; 16] = [
    0x38, 0x05, 0x8d, 0xaf, 0xb2, 0xcd, 0x48, 0x32, 0x90, 0x2a, 0x83, 0x58, 0x3a, 0xc0, 0x7e, 0x28,
];

#[test]
fn deserialize_str() {
    let uuid = Uuid::from_str(UUID).unwrap().readable();
    serde_test::assert_de_tokens(&uuid, &[Token::Str(UUID)]);
}

#[test]
fn deserialize_bytes() {
    let uuid = Uuid::from_slice(&UUID_BYTES).unwrap();
    serde_test::assert_de_tokens(&uuid.compact(), &[Token::Bytes(&UUID_BYTES)]);
    serde_test::assert_de_tokens(&uuid.readable(), &[Token::Bytes(&UUID_BYTES)]);
}

#[test]
fn deserialize_byte_array() {
    let uuid = Uuid::from_slice(&UUID_BYTES).unwrap();
    serde_test::assert_de_tokens(
        &uuid.readable(),
        &[
            Token::Seq { len: Some(16) },
            Token::U8(UUID_BYTES[0]),
            Token::U8(UUID_BYTES[1]),
            Token::U8(UUID_BYTES[2]),
            Token::U8(UUID_BYTES[3]),
            Token::U8(UUID_BYTES[4]),
            Token::U8(UUID_BYTES[5]),
            Token::U8(UUID_BYTES[6]),
            Token::U8(UUID_BYTES[7]),
            Token::U8(UUID_BYTES[8]),
            Token::U8(UUID_BYTES[9]),
            Token::U8(UUID_BYTES[10]),
            Token::U8(UUID_BYTES[11]),
            Token::U8(UUID_BYTES[12]),
            Token::U8(UUID_BYTES[13]),
            Token::U8(UUID_BYTES[14]),
            Token::U8(UUID_BYTES[15]),
            Token::SeqEnd,
        ],
    );

    serde_test::assert_de_tokens_error::<Compact<Uuid>>(
        &[Token::Seq { len: Some(16) }],
        "invalid type: sequence, expected bytes",
    );
}

#[test]
fn serialize_uuid() {
    let uuid = Uuid::from_slice(&UUID_BYTES).unwrap();
    serde_test::assert_ser_tokens(&uuid, &[Token::Str(UUID)]);
}
