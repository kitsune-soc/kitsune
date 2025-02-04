use std::io::{self, Write};

#[divan::bench]
fn normal_uuid(b: divan::Bencher<'_, '_>) {
    let uuid = uuid::Uuid::new_v4();
    b.bench(|| write!(io::empty(), "{uuid}"));
}

#[divan::bench]
fn simd_uuid(b: divan::Bencher<'_, '_>) {
    let uuid = speedy_uuid::Uuid::new_v4();
    b.bench(|| write!(io::empty(), "{uuid}"));
}

fn main() {
    divan::main();
}
