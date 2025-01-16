use self::fixtures::{Fixture, RNG_SEED};

mod fixtures;

#[futures_test::test]
async fn success_basic() {
    fastrand::seed(RNG_SEED);
    let fixtures = Fixture::generate();

    todo!();
}
