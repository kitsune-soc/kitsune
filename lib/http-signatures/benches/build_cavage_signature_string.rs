use criterion::{criterion_group, criterion_main, Criterion};
use http::{Method, Request, Uri};
use std::hint::black_box;

const CAVAGE_HEADER: &str =
    r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx""#;

fn build_cavage_signature_string1(c: &mut Criterion) {
    let signature_header = http_signatures::cavage::parse(CAVAGE_HEADER).unwrap();
    let request = Request::builder()
        .method(Method::GET)
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
        .unwrap();

    c.bench_function("build_cavage_signature_string", |b| {
        b.iter(|| {
            let _ = black_box(http_signatures::cavage::signature_string::construct(
                black_box(&request),
                black_box(&signature_header),
            ));
        });
    });
}

criterion_group!(
    build_cavage_signature_string,
    build_cavage_signature_string1
);
criterion_main!(build_cavage_signature_string);
