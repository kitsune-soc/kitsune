use komainu::params::ParamStorage;
use serde_test::Token;

#[test]
fn insert_get_works() {
    let mut map = ParamStorage::new();
    map.insert("hello", "world");
    assert_eq!(map.get("hello"), Some(&"world"));
}

#[test]
fn multi_insert_empty() {
    let mut map = ParamStorage::new();
    map.insert("hello", "world");
    map.insert("hello", "owo");
    assert_eq!(map.get("hello"), None);

    map.insert("hello", "uwu");
    assert_eq!(map.get("hello"), None);
}

#[test]
fn deserialize_impl() {
    let mut map1 = ParamStorage::new();
    map1.insert("hello", "world");

    serde_test::assert_de_tokens(
        &map1,
        &[
            Token::Map { len: Some(1) },
            Token::BorrowedStr("hello"),
            Token::BorrowedStr("world"),
            Token::MapEnd,
        ],
    );

    let mut map2 = ParamStorage::new();
    map2.insert("hello", "world");
    map2.insert("hello", "owo");

    assert!(map2.get("hello").is_none());

    serde_test::assert_de_tokens(
        &map2,
        &[
            Token::Map { len: Some(2) },
            Token::BorrowedStr("hello"),
            Token::BorrowedStr("world"),
            Token::BorrowedStr("hello"),
            Token::BorrowedStr("owo"),
            Token::MapEnd,
        ],
    );
}
