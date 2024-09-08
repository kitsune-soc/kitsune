use fast_cjson::CanonicalFormatter;
use serde::{Deserialize, Serialize};
use sonic_rs::{LazyValue, Serializer};
use std::io;

/// Small wrapper around the `sonic_rs` json! macro to encode the value as canonical JSON.
macro_rules! encode {
    (@raw $($tt:tt)+) => {
        (|v: sonic_rs::Value| -> io::Result<Vec<u8>> {
            let mut buf = Vec::new();
            let mut ser = Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
            v.serialize(&mut ser)?;
            Ok(buf)
        })($($tt)+)
    };

    ($($tt:tt)+) => {
        encode!(@raw sonic_rs::json!($($tt)+))
    };
}

/// These smoke tests come from securesystemslib, the library used by the TUF reference
/// implementation.
///
/// `<https://github.com/secure-systems-lab/securesystemslib/blob/f466266014aff529510216b8c2f8c8f39de279ec/tests/test_formats.py#L354-L389>`
#[test]
fn securesystemslib_asserts() -> io::Result<()> {
    assert_eq!(encode!([1, 2, 3])?, b"[1,2,3]");
    assert_eq!(encode!([1, 2, 3])?, b"[1,2,3]");
    assert_eq!(encode!([])?, b"[]");
    assert_eq!(encode!({})?, b"{}");
    assert_eq!(encode!({"A": [99]})?, br#"{"A":[99]}"#);
    assert_eq!(encode!({"A": true})?, br#"{"A":true}"#);
    assert_eq!(encode!({"B": false})?, br#"{"B":false}"#);
    assert_eq!(encode!({"x": 3, "y": 2})?, br#"{"x":3,"y":2}"#);
    assert_eq!(encode!({"x": 3, "y": null})?, br#"{"x":3,"y":null}"#);

    // Test conditions for invalid arguments.
    assert!(encode!(8.0).is_err());
    assert!(encode!({"x": 8.0}).is_err());

    Ok(())
}

/// Canonical JSON prints literal ASCII control characters instead of escaping them. Check
/// ASCII 0x00 - 0x1f, plus backslash and double quote (the only escaped characters).
///
/// The accepted strings were validated with securesystemslib, commit
/// f466266014aff529510216b8c2f8c8f39de279ec.
///
/// ```python
/// import securesystemslib.formats
/// encode = securesystemslib.formats.encode_canonical
/// for c in range(0x20):
///     print(repr(encode(chr(c))))
/// print(repr(encode('\\')))
/// print(repr(encode('"')))
/// ```
///
/// This can be a little difficult to wrap a mental string parser around. But you can verify
/// that all the control characters result in a 3-byte JSON string:
///
/// ```python
/// >>> all(map(lambda c: len(encode(chr(c))) == 3, range(0x20)))
/// True
/// ```
#[test]
fn ascii_control_characters() -> io::Result<()> {
    assert_eq!(encode!("\x00")?, b"\"\x00\"");
    assert_eq!(encode!("\x01")?, b"\"\x01\"");
    assert_eq!(encode!("\x02")?, b"\"\x02\"");
    assert_eq!(encode!("\x03")?, b"\"\x03\"");
    assert_eq!(encode!("\x04")?, b"\"\x04\"");
    assert_eq!(encode!("\x05")?, b"\"\x05\"");
    assert_eq!(encode!("\x06")?, b"\"\x06\"");
    assert_eq!(encode!("\x07")?, b"\"\x07\"");
    assert_eq!(encode!("\x08")?, b"\"\x08\"");
    assert_eq!(encode!("\x09")?, b"\"\x09\"");
    assert_eq!(encode!("\x0a")?, b"\"\x0a\"");
    assert_eq!(encode!("\x0b")?, b"\"\x0b\"");
    assert_eq!(encode!("\x0c")?, b"\"\x0c\"");
    assert_eq!(encode!("\x0d")?, b"\"\x0d\"");
    assert_eq!(encode!("\x0e")?, b"\"\x0e\"");
    assert_eq!(encode!("\x0f")?, b"\"\x0f\"");
    assert_eq!(encode!("\x10")?, b"\"\x10\"");
    assert_eq!(encode!("\x11")?, b"\"\x11\"");
    assert_eq!(encode!("\x12")?, b"\"\x12\"");
    assert_eq!(encode!("\x13")?, b"\"\x13\"");
    assert_eq!(encode!("\x14")?, b"\"\x14\"");
    assert_eq!(encode!("\x15")?, b"\"\x15\"");
    assert_eq!(encode!("\x16")?, b"\"\x16\"");
    assert_eq!(encode!("\x17")?, b"\"\x17\"");
    assert_eq!(encode!("\x18")?, b"\"\x18\"");
    assert_eq!(encode!("\x19")?, b"\"\x19\"");
    assert_eq!(encode!("\x1a")?, b"\"\x1a\"");
    assert_eq!(encode!("\x1b")?, b"\"\x1b\"");
    assert_eq!(encode!("\x1c")?, b"\"\x1c\"");
    assert_eq!(encode!("\x1d")?, b"\"\x1d\"");
    assert_eq!(encode!("\x1e")?, b"\"\x1e\"");
    assert_eq!(encode!("\x1f")?, b"\"\x1f\"");

    assert_eq!(encode!({"\t": "\n"})?, b"{\"\t\":\"\n\"}");
    assert_eq!(encode!("\\")?, b"\"\\\\\"");
    assert_eq!(encode!("\"")?, b"\"\\\"\"");

    Ok(())
}

/// A more involved test than any of the above for olpc-cjson's core competency: ordering
/// things.
#[test]
fn ordered_nested_object() -> io::Result<()> {
    assert_eq!(
            encode!({
                "nested": {
                    "bad": true,
                    "good": false
                },
                "b": 2,
                "a": 1,
                "c": {
                    "h": {
                        "h": -5,
                        "i": 3
                    },
                    "a": null,
                    "x": {}
                }
            })?,
            br#"{"a":1,"b":2,"c":{"a":null,"h":{"h":-5,"i":3},"x":{}},"nested":{"bad":true,"good":false}}"#.to_vec(),
        );

    Ok(())
}

/// This test asserts that the canonical representation of some real-world data always comes
/// out the same.
#[test]
fn actual_tuf_signed() {
    #[allow(clippy::unreadable_literal)]
    let encode_result = encode!(
    {
      "signed": {
        "_type": "timestamp",
        "spec_version": "1.0.0",
        "version": 1604605512,
        "expires": "2020-11-12T19:45:12.613154979Z",
        "meta": {
          "snapshot.json": {
            "length": 1278,
            "hashes": {
              "sha256": "56c4ecc3b331f6154d9a5005f6e2978e4198cc8c3b79746c25a592043a2d83d4"
            },
            "version": 1604605512
          }
        }
      }
    }
    );

    let encoded = encode_result.unwrap();
    let expected: Vec<u8> = vec![
        123, 34, 115, 105, 103, 110, 101, 100, 34, 58, 123, 34, 95, 116, 121, 112, 101, 34, 58, 34,
        116, 105, 109, 101, 115, 116, 97, 109, 112, 34, 44, 34, 101, 120, 112, 105, 114, 101, 115,
        34, 58, 34, 50, 48, 50, 48, 45, 49, 49, 45, 49, 50, 84, 49, 57, 58, 52, 53, 58, 49, 50, 46,
        54, 49, 51, 49, 53, 52, 57, 55, 57, 90, 34, 44, 34, 109, 101, 116, 97, 34, 58, 123, 34,
        115, 110, 97, 112, 115, 104, 111, 116, 46, 106, 115, 111, 110, 34, 58, 123, 34, 104, 97,
        115, 104, 101, 115, 34, 58, 123, 34, 115, 104, 97, 50, 53, 54, 34, 58, 34, 53, 54, 99, 52,
        101, 99, 99, 51, 98, 51, 51, 49, 102, 54, 49, 53, 52, 100, 57, 97, 53, 48, 48, 53, 102, 54,
        101, 50, 57, 55, 56, 101, 52, 49, 57, 56, 99, 99, 56, 99, 51, 98, 55, 57, 55, 52, 54, 99,
        50, 53, 97, 53, 57, 50, 48, 52, 51, 97, 50, 100, 56, 51, 100, 52, 34, 125, 44, 34, 108,
        101, 110, 103, 116, 104, 34, 58, 49, 50, 55, 56, 44, 34, 118, 101, 114, 115, 105, 111, 110,
        34, 58, 49, 54, 48, 52, 54, 48, 53, 53, 49, 50, 125, 125, 44, 34, 115, 112, 101, 99, 95,
        118, 101, 114, 115, 105, 111, 110, 34, 58, 34, 49, 46, 48, 46, 48, 34, 44, 34, 118, 101,
        114, 115, 105, 111, 110, 34, 58, 49, 54, 48, 52, 54, 48, 53, 53, 49, 50, 125, 125,
    ];
    assert_eq!(expected, encoded);
}

#[test]
fn raw_value() {
    let encoded = encode!({
        "nested": {
            "bad": true,
            "good": false
        },
        "b": 2,
        "a": 1,
        "c": {
            "h": {
                "h": -5,
                "i": 3
            },
            "a": null,
            "x": {}
        }
    })
    .unwrap();

    #[derive(Deserialize, Serialize)]
    struct TestValue<'a> {
        #[serde(borrow)]
        a: LazyValue<'a>,

        #[serde(borrow)]
        b: LazyValue<'a>,

        #[serde(borrow)]
        c: LazyValue<'a>,

        #[serde(borrow)]
        nested: LazyValue<'a>,
    }

    let parsed: TestValue<'_> = sonic_rs::from_slice(&encoded).unwrap();
    let encoded = encode!(parsed).unwrap();

    assert_eq!(
        br#"{"a":{"$sonic_rs::LazyValue":"1"},"b":{"$sonic_rs::LazyValue":"2"},"c":{"$sonic_rs::LazyValue":"{\"a\":null,\"h\":{\"h\":-5,\"i\":3},\"x\":{}}"},"nested":{"$sonic_rs::LazyValue":"{\"bad\":true,\"good\":false}"}}"#,
        encoded.as_slice(),
    );
}

#[test]
fn accept_raw_integer() {
    let raw_number: sonic_rs::RawNumber = sonic_rs::from_str("8").unwrap();

    let mut buf = Vec::new();
    let mut ser = Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
    assert!(raw_number.serialize(&mut ser).is_ok());
}

#[test]
fn reject_raw_float() {
    let raw_number: sonic_rs::RawNumber = sonic_rs::from_str("8.0").unwrap();
    let raw_number_small: sonic_rs::RawNumber = sonic_rs::from_str("12.3e+11").unwrap();
    let raw_number_large: sonic_rs::RawNumber = sonic_rs::from_str("13.12E+161").unwrap();

    for number in &[raw_number, raw_number_small, raw_number_large] {
        let mut buf = Vec::new();
        let mut ser = Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
        assert!(number.serialize(&mut ser).is_err());
    }
}
