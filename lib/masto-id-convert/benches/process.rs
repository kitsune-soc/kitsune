#![allow(clippy::unreadable_literal)]

use divan::{black_box, black_box_drop};

#[divan::bench(name = "process integer 110368129515784116")]
fn process_u64() {
    black_box_drop(masto_id_convert::process_u64(black_box(110368129515784116)));
}

#[divan::bench(name = "process ASCII 110368129515784116")]
fn process_ascii() {
    black_box_drop(masto_id_convert::process(black_box("110368129515784116")));
}

fn main() {
    divan::main();
}
