#![cfg(feature = "redis")]

use redis::ToRedisArgs;
use speedy_uuid::Uuid;
use std::str::FromStr;

const UUID: &str = "38058daf-b2cd-4832-902a-83583ac07e28";

#[test]
fn encode_redis() {
    let uuid = Uuid::from_str(UUID).unwrap();

    let mut buffer = Vec::new();
    uuid.write_redis_args(&mut buffer);

    assert_eq!(buffer, [UUID.as_bytes()]);
}
