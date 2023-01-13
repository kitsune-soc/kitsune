use http::HeaderValue;
use kitsune_http_signatures::HttpVerifier;

mod data;

// This is a broken test vector
// It should produce an error according to the specification, but according to the test vectors it should just omit the "(created)"/"(expires)" pseudo-headers
//#[tokio::test]
async fn all_headers() {
    let mut parts = self::data::get_parts();
    parts.headers.insert(
        "Signature",
        HeaderValue::from_static(
            r#"keyId="Test",algorithm="rsa-sha256",created=1402170695, expires=1402170699,headers="(request-target) (created) (expires) host date content-type digest content-length",signature="vSdrb+dS3EceC9bcwHSo4MlyKS59iFIrhgYkz8+oVLEEzmYZZvRs8rgOp+63LEM3v+MFHB32NfpB2bEKBIvB1q52LaEUHFv120V01IL+TAD48XaERZFukWgHoBTLMhYS2Gb51gWxpeIq8knRmPnYePbF5MOkR0Zkly4zKH7s1dE=""#
        ),
    );
    let public_key = self::data::get_public_key();
    let signer = HttpVerifier::builder()
        .parts(&parts)
        .check_expiration(false)
        .build()
        .unwrap();

    signer
        .verify(move |key_id| {
            let key_id = key_id.to_string();

            async move {
                assert_eq!(key_id, "Test");
                Ok(public_key)
            }
        })
        .await
        .unwrap();
}
