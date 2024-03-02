#![allow(unsafe_code)]

use self::wasi::logging::logging::{self, Level};

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

        Ok(activity)
    }
}

export!(Mrf);
