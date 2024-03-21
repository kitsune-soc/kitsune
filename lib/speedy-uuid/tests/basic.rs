use speedy_uuid::Uuid;
use std::str::FromStr;

const UUID: &str = "38058daf-b2cd-4832-902a-83583ac07e28";

#[test]
fn roundtrip() {
    let uuid = Uuid::from_str(UUID).unwrap();
    assert_eq!(UUID, uuid.to_string());
}
