use divan::{black_box, black_box_drop};
use http_signatures::cavage;

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

const CAVAGE_HEADER_1: &str = r#"keyId="Test",algorithm="rsa-sha256",headers="(request-target) host date",signature="qdx+H7PHHDZgy4y/Ahn9Tny9V3GP6YgBPyUXMmoxWtLbHpUnXS2mg2+SbrQDMCJypxBLSPQR2aAjn7ndmw2iicw3HMbe8VfEdKFYRqzic+efkb3nndiv/x1xSHDJWeSWkx3ButlYSuBskLu6kd9Fswtemr3lgdDEmn04swr2Os0=""#;
const CAVAGE_HEADER_2: &str = r#"keyId="Test",algorithm="rsa-sha256",created=1402170695, expires=1402170699,headers="(request-target) (created) (expires) host date content-type digest content-length",signature="vSdrb+dS3EceC9bcwHSo4MlyKS59iFIrhgYkz8+oVLEEzmYZZvRs8rgOp+63LEM3v+MFHB32NfpB2bEKBIvB1q52LaEUHFv120V01IL+TAD48XaERZFukWgHoBTLMhYS2Gb51gWxpeIq8knRmPnYePbF5MOkR0Zkly4zKH7s1dE=""#;

#[divan::bench]
fn parse_cavage_header1() {
    black_box_drop(cavage::parse(black_box(CAVAGE_HEADER_1)));
}

#[divan::bench]
fn parse_cavage_header2() {
    black_box_drop(cavage::parse(black_box(CAVAGE_HEADER_2)));
}

fn main() {
    divan::main();
}
