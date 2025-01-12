use komainu::flow::{self, pkce};
use sha2::{Digest, Sha256};
use std::borrow::Cow;

#[test]
fn verify_rfc_payload_s256() {
    let verifier = [
        116, 24, 223, 180, 151, 153, 224, 37, 79, 250, 96, 125, 216, 173, 187, 186, 22, 212, 37,
        77, 105, 214, 191, 240, 91, 88, 5, 88, 83, 132, 141, 121,
    ];

    let verifier_base64 = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";
    assert_eq!(
        base64_simd::URL_SAFE_NO_PAD.encode_to_string(verifier),
        verifier_base64
    );

    let challenge = [
        19, 211, 30, 150, 26, 26, 216, 236, 47, 22, 177, 12, 76, 152, 46, 8, 118, 168, 120, 173,
        109, 241, 68, 86, 110, 225, 137, 74, 203, 112, 249, 195,
    ];

    let challenge_base64 = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";
    assert_eq!(
        base64_simd::URL_SAFE_NO_PAD.encode_to_string(challenge),
        challenge_base64
    );

    let payload = pkce::Payload {
        method: pkce::Method::S256,
        challenge: Cow::Borrowed(challenge_base64),
    };
    payload.verify(verifier_base64).unwrap();
}

#[test]
fn verify_none() {
    let challenge = "dr.pebber";
    let payload = pkce::Payload {
        method: pkce::Method::None,
        challenge: Cow::Borrowed(challenge),
    };
    payload.verify(challenge).unwrap();
}

#[test]
fn none_reject_different() {
    let challenge = "fanter";
    let verifier = "monsert";

    let payload = pkce::Payload {
        method: pkce::Method::None,
        challenge: Cow::Borrowed(challenge),
    };

    payload.verify(challenge).unwrap();
    assert!(matches!(
        payload.verify(verifier).unwrap_err(),
        flow::Error::InvalidGrant
    ));
}

#[test]
fn s256_reject_different() {
    let challenge = "cokey cola";
    let verifier = "spirte";

    let encoded_challenge =
        base64_simd::URL_SAFE_NO_PAD.encode_to_string(Sha256::digest(challenge));

    let payload = pkce::Payload {
        method: pkce::Method::S256,
        challenge: Cow::Borrowed(&encoded_challenge),
    };

    payload.verify(challenge).unwrap();
    assert!(matches!(
        payload.verify(verifier).unwrap_err(),
        flow::Error::InvalidGrant
    ));
}
