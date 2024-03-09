use criterion::{criterion_group, criterion_main, Criterion};
use http_signatures::cavage;
use std::hint::black_box;

const CAVAGE_HEADER_1: &str = r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#;
const CAVAGE_HEADER_2: &str = r#"keyId="Test",algorithm="rsa-sha256",created=1402170695, expires=1402170699,headers="(request-target) (created) (expires) host date content-type digest content-length",signature="vSdrb+dS3EceC9bcwHSo4MlyKS59iFIrhgYkz8+oVLEEzmYZZvRs8rgOp+63LEM3v+MFHB32NfpB2bEKBIvB1q52LaEUHFv120V01IL+TAD48XaERZFukWgHoBTLMhYS2Gb51gWxpeIq8knRmPnYePbF5MOkR0Zkly4zKH7s1dE=""#;

fn header1(c: &mut Criterion) {
    c.bench_function("parse_cavage_header1", |b| {
        b.iter(|| {
            let _ = black_box(cavage::parse(black_box(CAVAGE_HEADER_1)));
        });
    });
}

fn header2(c: &mut Criterion) {
    c.bench_function("parse_cavage_header2", |b| {
        b.iter(|| {
            let _ = black_box(cavage::parse(black_box(CAVAGE_HEADER_2)));
        });
    });
}

criterion_group!(parse_cavage_header, header1, header2);
criterion_main!(parse_cavage_header);
