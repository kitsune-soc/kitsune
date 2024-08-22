use proptest::{prop_assert_eq, proptest};
use proptest_derive::Arbitrary;
use serde::Serialize;

#[derive(Arbitrary, Debug, Serialize)]
pub struct Data {
    int: i32,
    int_array: Vec<i32>,
    string: String,
    string_array: Vec<String>,
    big_int: i128,
}

proptest! {
    #[test]
    fn test_serialization(data: Data) {
        let mut olpc_cjson = Vec::new();
        let mut olpc_cjson_ser = serde_json::Serializer::with_formatter(&mut olpc_cjson, olpc_cjson::CanonicalFormatter::new());
        data.serialize(&mut olpc_cjson_ser).unwrap();

        let mut our_cjson = Vec::new();
        let mut our_cjson_ser = sonic_rs::Serializer::with_formatter(&mut our_cjson, fast_cjson::CanonicalFormatter::new());
        data.serialize(&mut our_cjson_ser).unwrap();

        prop_assert_eq!(olpc_cjson, our_cjson);
    }
}
