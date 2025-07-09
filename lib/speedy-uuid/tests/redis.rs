#![cfg(feature = "redis")]

use fred::types::{FromValue, Value};
use speedy_uuid::Uuid;
use std::str::FromStr;

const UUID: &str = "38058daf-b2cd-4832-902a-83583ac07e28";
const UUID_BYTES: [u8; 16] = [
    0x38, 0x05, 0x8d, 0xaf, 0xb2, 0xcd, 0x48, 0x32, 0x90, 0x2a, 0x83, 0x58, 0x3a, 0xc0, 0x7e, 0x28,
];

#[test]
fn encode_redis() {
    let uuid = Uuid::from_str(UUID).unwrap();
    let redis_value = Value::from(uuid);

    assert!(matches!(redis_value, Value::String(..)));
    assert_eq!(redis_value.as_str().as_deref(), Some(UUID));
}

#[test]
fn decode_redis() {
    let uuid = Uuid::from_slice(&UUID_BYTES).unwrap();

    let decoded = Uuid::from_value(Value::from_static(&UUID_BYTES)).unwrap();
    assert_eq!(uuid, decoded);

    let decoded = Uuid::from_value(Value::from_static_str(UUID)).unwrap();
    assert_eq!(uuid, decoded);

    let result = Uuid::from_value(Value::Array(vec![]));
    assert!(result.is_err());
}
