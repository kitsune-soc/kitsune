use criterion::{criterion_group, criterion_main, Criterion};
use http_signatures::cavage;
use std::hint::black_box;

const CAVAGE_HEADER: &str = r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#;

fn header1(c: &mut Criterion) {
    c.bench_function("parse_cavage_header1", |b| {
        b.iter(|| {
            let _ = cavage::parse(black_box(CAVAGE_HEADER));
        });
    });
}

criterion_group!(parse_cavage_header, header1);
criterion_main!(parse_cavage_header);
