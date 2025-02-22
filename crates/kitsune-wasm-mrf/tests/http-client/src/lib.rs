#![allow(unsafe_code, unsafe_op_in_unsafe_fn)]

use self::fep::mrf::http_client;
use std::str;

wit_bindgen::generate!({
    with: {
        "wasi:logging/logging": generate
    }
});

struct Mrf;

impl Guest for Mrf {
    fn config_schema() -> Option<String> {
        None
    }

    fn transform(
        _configuration: String,
        _direction: Direction,
        activity: String,
    ) -> Result<String, Error> {
        let request = http_client::Request {
            url: "https://aumetra.xyz/blog".into(),
            method: "GET".into(),
            headers: vec![],
            body: None,
        };
        let response = http_client::do_request(&request).unwrap();

        assert_eq!(response.status, 200);
        let body = response.body.next().unwrap().unwrap();
        assert_eq!(response.body.next().unwrap(), None);

        let body_str = str::from_utf8(&body).unwrap();
        assert_eq!(body_str, "[response here]");

        Ok(activity)
    }
}

export!(Mrf);
