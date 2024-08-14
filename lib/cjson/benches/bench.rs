use divan::black_box;
use mimalloc::MiMalloc;
use serde::Serialize;
use std::fs;

#[global_allocator]
static ALLOC: divan::AllocProfiler<MiMalloc> = divan::AllocProfiler::new(MiMalloc);

fn load_data() -> sonic_rs::Value {
    let bench_data = fs::read_to_string("benches/bench_512kb.json").unwrap();
    sonic_rs::from_str(&bench_data).unwrap()
}

#[divan::bench]
fn canonicalize(b: divan::Bencher<'_, '_>) {
    let data = load_data();

    b.bench(|| {
        let mut buf = Vec::new();
        let mut ser =
            sonic_rs::Serializer::with_formatter(&mut buf, cjson::CanonicalFormatter::new());
        black_box(&data).serialize(&mut ser).unwrap();
        buf
    });
}

#[divan::bench]
fn canonicalize_olpc_cjson(b: divan::Bencher<'_, '_>) {
    let data = load_data();

    b.bench(|| {
        let mut buf = Vec::new();
        let mut ser =
            serde_json::Serializer::with_formatter(&mut buf, olpc_cjson::CanonicalFormatter::new());
        black_box(&data).serialize(&mut ser).unwrap();
        buf
    });
}

fn main() {
    divan::main();
}
