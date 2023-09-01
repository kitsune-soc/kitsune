use criterion::{
    async_executor::FuturesExecutor, black_box, criterion_group, criterion_main, Criterion,
};
use futures_util::future;

const SIMPLE_POST: &str = "Hello, #World! This is a benchmark for the post transformer of @kitsune";

fn simple_bench(c: &mut Criterion) {
    c.bench_function("simple_transform", |b| {
        b.to_async(FuturesExecutor).iter(|| async {
            let _ = post_process::transform(black_box(SIMPLE_POST), future::ok).await;
        })
    });
}

criterion_group!(simple, simple_bench);
criterion_main!(simple);
