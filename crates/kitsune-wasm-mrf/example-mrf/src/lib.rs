use self::exports::fep::mrf::{meta, transform};

wit_bindgen::generate!({
    world: "mrf",
    exports: {
        "fep:mrf/meta": Mrf,
        "fep:mrf/transform": Mrf,
    }
});

struct Mrf;

impl meta::Guest for Mrf {
    fn name() -> String {
        "Example MRF".into()
    }

    fn version() -> String {
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
