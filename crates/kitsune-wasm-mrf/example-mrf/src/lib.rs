#![allow(unsafe_code)]

use self::{
    fep::mrf::keyvalue::Bucket,
    wasi::logging::logging::{self, Level},
};
use rand::{distributions::Alphanumeric, Rng};

wit_bindgen::generate!({
    with: {
        "wasi:logging/logging": generate
    }
});

fn generate_random_key() -> String {
    rand::thread_rng()
        .sample_iter(Alphanumeric)
        .take(50)
        .map(|byte| byte as char)
        .collect()
}

struct Mrf;

impl Guest for Mrf {
    fn transform(
        _config: String,
        _direction: Direction,
        activity: String,
    ) -> Result<String, Error> {
        logging::log(
            Level::Debug,
            "example-mrf",
            "we got an activity! that's cool!",
        );

        // We even have a key-value store! Check this out:
        let key = generate_random_key();
        let bucket = Bucket::open("example-bucket").unwrap();

        bucket.set(&key, b"world").unwrap();

        assert!(bucket.exists(&key).unwrap());
        assert_eq!(bucket.get(&key).unwrap(), Some(b"world".to_vec()));

        bucket.delete(&key).unwrap();

        Ok(activity)
    }
}

export!(Mrf);
