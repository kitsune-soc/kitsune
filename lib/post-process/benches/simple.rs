use criterion::{async_executor::FuturesExecutor, criterion_group, criterion_main, Criterion};
use futures::future;
use std::hint::black_box;

const SIMPLE_POST: &str = "Hello, #World! This is a benchmark for the post transformer of @kitsune";

fn simple_bench(c: &mut Criterion) {
    c.bench_function("simple_transform", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            post_process::transform(black_box(SIMPLE_POST), black_box(future::ok)).await
        });
    });
}

criterion_group!(simple, simple_bench);
criterion_main!(simple);
