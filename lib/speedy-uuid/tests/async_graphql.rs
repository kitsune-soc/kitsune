#![cfg(feature = "async-graphql")]

use async_graphql::{ScalarType, connection::CursorType};
use speedy_uuid::Uuid;

const UUID: &str = "38058daf-b2cd-4832-902a-83583ac07e28";
const UUID_BYTES: [u8; 16] = [
    0x38, 0x05, 0x8d, 0xaf, 0xb2, 0xcd, 0x48, 0x32, 0x90, 0x2a, 0x83, 0x58, 0x3a, 0xc0, 0x7e, 0x28,
];

#[test]
fn cursor_encode_decode() {
    let parsed_cursor: Uuid = CursorType::decode_cursor(UUID).unwrap();
    assert_eq!(parsed_cursor.as_bytes(), &UUID_BYTES);

    let encoded_cursor = CursorType::encode_cursor(&parsed_cursor);
    assert_eq!(encoded_cursor, UUID);
}

#[test]
fn cursor_invalid_input() {
    let result: Result<Uuid, speedy_uuid::Error> = CursorType::decode_cursor("NOT A UUID");
    assert!(result.is_err());
}

#[test]
fn scalar_encode_decode() {
    let parsed_scalar: Uuid = ScalarType::parse(async_graphql::Value::String(UUID.into())).unwrap();
    assert_eq!(parsed_scalar.as_bytes(), &UUID_BYTES);

    let encoded_scalar = ScalarType::to_value(&parsed_scalar);
    assert_eq!(encoded_scalar, async_graphql::Value::String(UUID.into()));
}

#[test]
fn scalar_invalid_input() {
    let result: async_graphql::InputValueResult<Uuid> =
        ScalarType::parse(async_graphql::Value::Null);
    assert!(result.is_err());

    let result: async_graphql::InputValueResult<Uuid> =
        ScalarType::parse(async_graphql::Value::String("NOT A UUID".into()));
    assert!(result.is_err());
}
