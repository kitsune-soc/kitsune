use athena::{Keepable, KeeperOfTheSecrets};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
struct CustomStruct {
    number: u32,
}

#[typetag::serde]
impl Keepable for CustomStruct {}

#[test]
fn roundtrip() {
    let original = CustomStruct { number: 1312 };

    let keeper = KeeperOfTheSecrets::new(original.clone());
    let value = keeper.get::<CustomStruct>().unwrap();

    assert_eq!(original, *value);
}

#[test]
fn downcasting_into_other() {
    let keeper = KeeperOfTheSecrets::new(CustomStruct { number: 161 });

    assert_eq!(
        keeper.get::<CustomStruct>(),
        Some(CustomStruct { number: 161 }).as_ref()
    );

    assert!(keeper.get::<()>().is_none());
    assert!(keeper.get::<u32>().is_none());
    assert!(keeper.get::<String>().is_none());
}
