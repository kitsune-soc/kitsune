use divan::{black_box, Bencher};
use std::time::{Duration, SystemTime};
use tick_tock_mock::{Clock, DeltaDirection};

#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

#[divan::bench]
fn std_systemtime_now() -> SystemTime {
    black_box(SystemTime::now())
}

#[divan::bench]
fn ttm_now_tl() -> SystemTime {
    black_box(tick_tock_mock::now())
}

#[divan::bench]
fn ttm_now_simple(bencher: Bencher<'_, '_>) {
    let clock = Clock::new();
    bencher.bench(|| black_box(clock.now()));
}

#[divan::bench]
fn ttm_now_mock(bencher: Bencher<'_, '_>) {
    let (clock, mock) = Clock::mockable();
    mock.adjust(DeltaDirection::Add, Duration::from_secs(1));

    bencher.bench(|| black_box(clock.now()));
}

fn main() {
    divan::main();
}
