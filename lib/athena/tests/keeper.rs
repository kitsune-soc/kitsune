use athena::{Keepable, KeeperOfTheSecrets};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct CustomStruct {
    number: u32,
}

#[typetag::serde]
impl Keepable for CustomStruct {}

#[derive(Deserialize, Serialize)]
struct OtherCustomStruct {
    other: u32,
}

#[typetag::serde]
impl Keepable for OtherCustomStruct {}

#[test]
fn roundtrip() {
    let original = CustomStruct { number: 1312 };

    let keeper = KeeperOfTheSecrets::new(original.clone());
    let value = keeper.get::<CustomStruct>().unwrap();

    assert_eq!(original, *value);
}

#[test]
fn serialize_roundtrip() {
    let original = CustomStruct { number: 1312 };

    let keeper: KeeperOfTheSecrets = {
        let keeper = KeeperOfTheSecrets::new(original.clone());
        postcard::from_bytes(&postcard::to_allocvec(&keeper).unwrap()).unwrap()
    };
    let value = keeper.get::<CustomStruct>().unwrap();

    assert_eq!(original, *value);
}

#[test]
fn downcasting_into_other() {
    let value = CustomStruct { number: 161 };
    let keeper = KeeperOfTheSecrets::new(value.clone());

    assert!(keeper.get::<OtherCustomStruct>().is_none());
    assert_eq!(keeper.get::<CustomStruct>(), Some(&value));
}
