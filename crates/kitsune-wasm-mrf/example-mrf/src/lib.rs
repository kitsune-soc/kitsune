mod transform {
    wit_bindgen::generate!({
        world: "mrf-v1",
        exports: {
            world: super::Mrf,
        }
    });
}

struct Mrf;

impl transform::Guest for Mrf {
    fn transform(
        _config: String,
        _direction: transform::Direction,
        activity: String,
    ) -> Result<String, transform::Error> {
        // We could do a lot here. But this is just an example
        // So we do literally nothing. Just wasting execution time.
        Ok(activity)
    }
}
