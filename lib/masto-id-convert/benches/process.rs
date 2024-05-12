#![allow(clippy::unreadable_literal)]

use divan::black_box_drop;

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

#[divan::bench(name = "process integer", args = [110368129515784116])]
fn process_u64(id: u64) {
    black_box_drop(masto_id_convert::process_u64(id));
}

#[divan::bench(name = "process ASCII", args = ["110368129515784116"])]
fn process_ascii(id: &str) {
    black_box_drop(masto_id_convert::process(id));
}

fn main() {
    divan::main();
}
