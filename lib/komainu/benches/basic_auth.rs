#[global_allocator]
static GLOBAL: divan::AllocProfiler = divan::AllocProfiler::system();

#[divan::bench_group]
mod headers {
    use divan::{black_box, black_box_drop};
    use headers::{authorization::Basic, Authorization, HeaderMapExt};

    #[divan::bench]
    fn rfc_value(b: divan::Bencher<'_, '_>) {
        let mut map = http::HeaderMap::new();
        map.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="),
        );

        b.bench(|| {
            let auth = black_box(&map).typed_get::<Authorization<Basic>>().unwrap();
            black_box_drop((auth.username(), auth.password()));
            auth
        });
    }
}

#[divan::bench_group]
mod ours {
    use divan::{black_box, black_box_drop};
    use komainu::extract::BasicAuth;

    #[divan::bench]
    fn rfc_value(b: divan::Bencher<'_, '_>) {
        let mut map = http::HeaderMap::new();
        map.insert(
            http::header::AUTHORIZATION,
            http::HeaderValue::from_static("Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="),
        );

        b.bench(|| {
            let auth = BasicAuth::extract(black_box(&map)).unwrap();
            black_box_drop((auth.username(), auth.password()));
            auth
        });
    }
}

fn main() {
    divan::main();
}
