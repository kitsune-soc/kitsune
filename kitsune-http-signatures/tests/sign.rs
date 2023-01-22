use self::data::{get_parts, get_private_key};
use kitsune_http_signatures::{HttpSigner, HttpVerifier, PrivateKey, SignatureComponent};
use ring::signature::{KeyPair, UnparsedPublicKey, RSA_PKCS1_2048_8192_SHA256};

mod data;

#[tokio::test]
async fn sign_some() {
    let private_key = get_private_key();
    let public_key = UnparsedPublicKey::new(
        &RSA_PKCS1_2048_8192_SHA256,
        private_key.public_key().clone(),
    );
    let private_key = PrivateKey::builder()
        .key_id("Test")
        .key(private_key)
        .build()
        .unwrap();

    let mut parts = get_parts();
    let signer = HttpSigner::builder().build().unwrap();
    let (name, value) = signer
        .sign(
            &parts,
            vec![
                SignatureComponent::RequestTarget,
                SignatureComponent::Header("Digest"),
                SignatureComponent::Header("Date"),
            ],
            private_key,
        )
        .await
        .unwrap();

    assert_eq!(
        r#"keyId="Test",signature="HuuCVzbU3mZ2n2tluqlZRynK1GQSdJoO/vVk2hqZXgqoZKKxIqkFi2cyqygqT3Kr5va84iV8wEQF0ewPOW3cl6Qy11c0UaNDxuIhImCR6VzmrbZv0DXrtw3SlLjH2LccI0o0aoh9FairG2+lnDw5gGWZjIgQBDxQkIz5KH4D1HrLz2GnG2mhLuviLWrV+bQ4dBO8yOvQrZjqKv8vMyIV/PVZuWN5A/N9f0lzxr+Oqsd3S88qBdY5t5h9t7vvqOyEeL6UEdQAchnNmVxCnc0oNb+axT7bsaRe5rJZhm6jcJW0nvhmWR/Qyfv6vcYtnzRFUcTeEcrgPGSJP13g1wXWxA==",headers="(request-target) Digest Date""#,
        value.to_str().unwrap()
    );

    parts.headers.insert(name, value);

    let verifier = HttpVerifier::builder().build().unwrap();
    verifier
        .verify(&parts, |key_id| {
            assert_eq!(key_id, "Test");
            async { Ok(public_key) }
        })
        .await
        .unwrap();
}
