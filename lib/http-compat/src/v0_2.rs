use crate::Compat;
use std::str::FromStr;

impl Compat for http02::HeaderName {
    type Output = http1::HeaderName;

    fn compat(self) -> Self::Output {
        http1::HeaderName::from_bytes(self.as_str().as_bytes()).unwrap()
    }
}

impl Compat for http02::HeaderValue {
    type Output = http1::HeaderValue;

    fn compat(self) -> Self::Output {
        http1::HeaderValue::from_bytes(self.as_bytes()).unwrap()
    }
}

impl Compat for http02::HeaderMap {
    type Output = http1::HeaderMap;

    fn compat(self) -> Self::Output {
        self.iter()
            .map(|(name, value)| (name.clone().compat(), value.clone().compat()))
            .collect()
    }
}

impl Compat for http02::Method {
    type Output = http1::Method;

    fn compat(self) -> Self::Output {
        http1::Method::from_bytes(self.as_str().as_bytes()).unwrap()
    }
}

impl<B> Compat for http02::Request<B> {
    type Output = http1::Request<B>;

    fn compat(self) -> Self::Output {
        let (parts, body) = self.into_parts();
        let mut req_builder = http1::Request::builder()
            .method(parts.method.compat())
            .uri(parts.uri.compat())
            .version(parts.version.compat());
        *req_builder.headers_mut().unwrap() = parts.headers.compat();

        req_builder.body(body).unwrap()
    }
}

impl Compat for http02::Uri {
    type Output = http1::Uri;

    fn compat(self) -> Self::Output {
        http1::Uri::from_str(&self.to_string()).unwrap()
    }
}

impl Compat for http02::Version {
    type Output = http1::Version;

    fn compat(self) -> Self::Output {
        match self {
            http02::Version::HTTP_09 => http1::Version::HTTP_09,
            http02::Version::HTTP_10 => http1::Version::HTTP_10,
            http02::Version::HTTP_11 => http1::Version::HTTP_11,
            http02::Version::HTTP_2 => http1::Version::HTTP_2,
            http02::Version::HTTP_3 => http1::Version::HTTP_3,
            _ => unreachable!(),
        }
    }
}
