mod meta {
    wit_bindgen::generate!({
        world: "meta",
        exports: {
            world: super::Mrf,
        }
    });
}

mod transform {
    wit_bindgen::generate!({
        world: "mrf-v1",
        exports: {
            world: super::Mrf,
        }
    });
}

struct Mrf;

impl meta::Guest for Mrf {
    fn name() -> String {
        "Example MRF".into()
    }

    fn version() -> String {
        "1.0.0".into()
    }

    fn api_version() -> String {
        "1.0.0".into()
    }
}

impl transform::Guest for Mrf {
    fn transform(
        _direction: transform::Direction,
        activity: String,
    ) -> Result<String, transform::Error> {
        // We could do a lot here. But this is just an example
        // So we do literally nothing. Just wasting execution time.
        Ok(activity)
    }
}
