use criterion::{criterion_group, criterion_main, Criterion};
use std::{
    hint::black_box,
    time::{Duration, SystemTime},
};
use tick_tock_mock::{Clock, DeltaDirection};

fn simple_now_mock(c: &mut Criterion) {
    let clock = Clock::new();
    c.bench_function("simple_now_mock", |b| {
        b.iter(|| black_box(clock.now()));
    });

    c.bench_function("simple_now_mock_tl", |b| {
        b.iter(|| black_box(tick_tock_mock::now()));
    });

    let (clock, mock) = Clock::mockable();
    mock.adjust(DeltaDirection::Add, Duration::from_secs(1));

    c.bench_function("simple_now_mock_adjust", |b| {
        b.iter(|| black_box(clock.now()));
    });
}

fn simple_now_std(c: &mut Criterion) {
    c.bench_function("simple_now_std", |b| {
        b.iter(|| black_box(SystemTime::now()));
    });
}

criterion_group!(simple_now, simple_now_mock, simple_now_std);
criterion_main!(simple_now);
