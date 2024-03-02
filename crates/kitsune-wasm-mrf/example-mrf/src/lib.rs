#![allow(clippy::missing_safety_doc, clippy::transmute_int_to_bool, unsafe_code)]

use self::{
    fep::mrf::keyvalue::{self, Bucket},
    wasi::logging::logging::{self, Level},
};

wit_bindgen::generate!();

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
        let bucket = Bucket::open_bucket("example-bucket").unwrap();
        keyvalue::set(&bucket, "hello", b"world").unwrap();
        assert!(keyvalue::exists(&bucket, "hello").unwrap());
        keyvalue::delete(&bucket, "hello").unwrap();

        Ok(activity)
    }
}

export!(Mrf);
