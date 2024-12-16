use komainu::flow::{PkceMethod, PkcePayload};
use std::borrow::Cow;

#[test]
fn verify_rfc_payload_s256() {
    let verifier = [
        116, 24, 223, 180, 151, 153, 224, 37, 79, 250, 96, 125, 216, 173, 187, 186, 22, 212, 37,
        77, 105, 214, 191, 240, 91, 88, 5, 88, 83, 132, 141, 121,
    ];

    let verifier_base64 = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk";

    let challenge = [
        19, 211, 30, 150, 26, 26, 216, 236, 47, 22, 177, 12, 76, 152, 46, 8, 118, 168, 120, 173,
        109, 241, 68, 86, 110, 225, 137, 74, 203, 112, 249, 195,
    ];

    let challenge_base64 = "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM";

    let payload = PkcePayload {
        method: PkceMethod::S256,
        challenge: Cow::Borrowed(challenge_base64),
    };
    payload.verify(verifier_base64).unwrap();
}
