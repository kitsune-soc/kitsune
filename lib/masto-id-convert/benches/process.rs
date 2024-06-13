#![allow(clippy::unreadable_literal)]

use uuid::Uuid;

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

#[divan::bench(name = "process integer", args = [110368129515784116])]
fn process_u64(id: u64) -> Uuid {
    masto_id_convert::process_u64(id)
}

#[divan::bench(name = "process ASCII", args = ["110368129515784116"])]
fn process_ascii(id: &str) -> Result<Uuid, masto_id_convert::Error> {
    masto_id_convert::process(id)
}

fn main() {
    divan::main();
}
