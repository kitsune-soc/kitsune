#![allow(clippy::missing_safety_doc, clippy::transmute_int_to_bool, unsafe_code)]

use self::{
    fep::mrf::keyvalue::{self, Bucket},
    wasi::logging::logging::{self, Level},
};
use rand::{distributions::Alphanumeric, Rng};

wit_bindgen::generate!();

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
        let bucket = Bucket::open_bucket("example-bucket").unwrap();

        keyvalue::set(&bucket, &key, b"world").unwrap();
        assert!(keyvalue::exists(&bucket, &key).unwrap());
        keyvalue::delete(&bucket, &key).unwrap();

        Ok(activity)
    }
}

export!(Mrf);
