use const_oid::db::rfc5912::RSA_ENCRYPTION;
use http_signatures::BoxError;
use pkcs8::{
    SubjectPublicKeyInfoRef,
    der::{Encode, asn1::BitStringRef},
    spki::AlgorithmIdentifier,
};
use scoped_futures::ScopedFutureExt;
use std::{future, time::Duration};
use tick_tock_mock::DeltaDirection;

mod data;

#[test]
fn basic_signature() {
    let req = self::data::get_request();
    let public_key = self::data::get_public_key();

    let signature_header = http_signatures::cavage::parse(r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#).unwrap();
    let signature_string =
        http_signatures::cavage::signature_string::construct(&req, &signature_header).unwrap();

    assert!(
        http_signatures::crypto::verify(
            signature_string.as_bytes(),
            signature_header.signature,
            &public_key
        )
        .is_ok()
    );
}

#[tokio::test]
async fn easy() {
    let req = self::data::get_request();
    let private_key = self::data::get_private_key();

    let signed_request =
        http_signatures::cavage::easy::sign(req, "Test", &self::data::get_pkcs8_private_key())
            .await
            .unwrap();

    http_signatures::cavage::easy::verify(&signed_request, |key_id| {
        assert_eq!(key_id, "Test");

        let public_key = private_key.public();
        let spki = SubjectPublicKeyInfoRef {
            algorithm: AlgorithmIdentifier {
                oid: RSA_ENCRYPTION,
                parameters: None,
            },
            subject_public_key: BitStringRef::from_bytes(public_key.as_ref()).unwrap(),
        };

        future::ready(spki.to_der()).scoped()
    })
    .await
    .unwrap();
}

#[tokio::test]
async fn easy_expires() {
    let (clock, mock) = tick_tock_mock::Clock::mockable();
    let _guard = clock.enter();

    let req = self::data::get_request();
    let signed_request =
        http_signatures::cavage::easy::sign(req, "Test", &self::data::get_pkcs8_private_key())
            .await
            .unwrap();

    // Forward the clock an hour..
    mock.adjust(DeltaDirection::Add, Duration::from_secs(60 * 60));

    http_signatures::cavage::easy::verify(&signed_request, |_key_id| {
        future::ready(
            #[allow(unreachable_code)]
            {
                unreachable!() as Result<_, BoxError>
            },
        )
        .scoped()
    })
    .await
    .unwrap_err();
}
