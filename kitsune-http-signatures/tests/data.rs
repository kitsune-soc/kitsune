use http::{request::Parts, Method, Request, Uri};
use pkcs8::{der::Decode, Document, SubjectPublicKeyInfo};
use ring::signature::{UnparsedPublicKey, RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY};

const PUBLIC_KEY: &str = r#"-----BEGIN PUBLIC KEY-----
MIGfMA0GCSqGSIb3DQEBAQUAA4GNADCBiQKBgQDCFENGw33yGihy92pDjZQhl0C3
6rPJj+CvfSC8+q28hxA161QFNUd13wuCTUcq0Qd2qsBe/2hFyc2DCJJg0h1L78+6
Z4UMR7EOcpfdUE9Hf3m/hs+FUR45uBJeDK1HSFHD8bHKD6kv8FPGfJTotc+2xjJw
oYi+1hqp1fIekaxsyQIDAQAB
-----END PUBLIC KEY-----"#;

pub fn get_parts() -> Parts {
    Request::builder()
        .method(Method::POST)
        .uri(Uri::from_static("/foo?param=value&pet=dog"))
        .header("Host", "example.com")
        .header("Date", "Sun, 05 Jan 2014 21:31:40 GMT")
        .header("Content-Type", "application/json")
        .header(
            "Digest",
            "SHA-256=X48E9qOokqqrvdts8nOJRJN3OWDUoyWxBf7kbu9DBPE=",
        )
        .header("Content-Length", "18")
        .body(())
        .unwrap()
        .into_parts()
        .0
}

pub fn get_public_key() -> UnparsedPublicKey<Vec<u8>> {
    let pem = pem::parse(PUBLIC_KEY).unwrap();
    let pub_key = Document::from_der(&pem.contents).unwrap();
    let pub_key: SubjectPublicKeyInfo<'_> = pub_key.decode_msg().unwrap();
    let pub_key = pub_key.subject_public_key.to_vec();

    UnparsedPublicKey::new(&RSA_PKCS1_1024_8192_SHA256_FOR_LEGACY_USE_ONLY, pub_key)
}
