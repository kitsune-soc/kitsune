use divan::{black_box, Bencher};
use http::{Method, Request, Uri};

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

const CAVAGE_HEADER: &str =
    r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx""#;

#[divan::bench]
fn build_cavage_signature_string(bencher: Bencher<'_, '_>) {
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

    bencher.bench(|| {
        http_signatures::cavage::signature_string::construct(
            black_box(&request),
            black_box(&signature_header),
        )
    });
}

fn main() {
    divan::main();
}
