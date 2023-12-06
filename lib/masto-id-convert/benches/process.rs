#![allow(clippy::unreadable_literal)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn process_bench(c: &mut Criterion) {
    c.bench_function("process integer 110368129515784116", |b| {
        b.iter(|| masto_id_convert::process_u64(black_box(110368129515784116)));
    });

    c.bench_function("process ASCII 110368129515784116", |b| {
        b.iter(|| masto_id_convert::process(black_box("110368129515784116")));
    });
}

criterion_group!(benches, process_bench);
criterion_main!(benches);
