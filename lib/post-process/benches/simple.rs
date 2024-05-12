use divan::black_box;
use futures_executor::block_on;
use std::future;

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

const SIMPLE_POST: &str = "Hello, #World! This is a benchmark for the post transformer of @kitsune";

#[divan::bench(args = [SIMPLE_POST])]
fn simple_bench(post: &str) -> post_process::Result<String> {
    block_on(post_process::transform(
        post,
        black_box(|item| future::ready(Ok(item))),
    ))
}

fn main() {
    divan::main();
}
