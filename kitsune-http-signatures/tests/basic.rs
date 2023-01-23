use http::HeaderValue;
use kitsune_http_signatures::HttpVerifier;

mod data;

#[tokio::test]
async fn basic_signature() {
    let mut parts = self::data::get_parts();
    parts.headers.insert(
        "Signature",
        HeaderValue::from_static(
            r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#
        ),
    );
    let public_key = self::data::get_public_key();
    let signer = HttpVerifier::builder()
        .enforce_expiration(None)
        .build()
        .unwrap();

    signer
        .verify(&parts, move |key_id| async move {
            assert_eq!(key_id, "Test");
            Ok(public_key)
        })
        .await
        .unwrap();
}
