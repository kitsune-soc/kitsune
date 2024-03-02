#![allow(unsafe_code)]

wit_bindgen::generate!();

struct Mrf;

impl Guest for Mrf {
    fn transform(
        _config: String,
        _direction: Direction,
        activity: String,
    ) -> Result<String, Error> {
        // We could do a lot here. But this is just an example
        // So we do literally nothing. Just wasting execution time.
        Ok(activity)
    }
}

export!(Mrf);
