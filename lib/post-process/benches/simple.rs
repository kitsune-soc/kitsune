use divan::black_box;
use futures_executor::block_on;
use std::future;

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

const SIMPLE_POST: &str = "Hello, #World! This is a benchmark for the post transformer of @kitsune";

#[divan::bench]
fn simple_bench() -> post_process::Result<String> {
    block_on(post_process::transform(
        black_box(SIMPLE_POST),
        black_box(|item| future::ready(Ok(item))),
    ))
}

fn main() {
    divan::main();
}
