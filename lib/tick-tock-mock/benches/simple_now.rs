#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

#[divan::bench_group]
mod std {
    use divan::black_box;
    use std::time::SystemTime;

    #[divan::bench]
    fn systemtime_now() -> SystemTime {
        black_box(SystemTime::now())
    }
}

#[divan::bench_group]
mod tick_tock_mock {
    use divan::{black_box, Bencher};
    use std::time::{Duration, SystemTime};
    use tick_tock_mock::{Clock, DeltaDirection};

    #[divan::bench]
    fn now_thread_local() -> SystemTime {
        black_box(tick_tock_mock::now())
    }

    #[divan::bench]
    fn now(bencher: Bencher<'_, '_>) {
        let clock = Clock::new();
        bencher.bench(|| black_box(&clock).now());
    }

    #[divan::bench]
    fn now_mocked(bencher: Bencher<'_, '_>) {
        let (clock, mock) = Clock::mockable();
        mock.adjust(DeltaDirection::Add, Duration::from_secs(1));

        bencher.bench(|| black_box(&clock).now());
    }
}

fn main() {
    divan::main();
}
