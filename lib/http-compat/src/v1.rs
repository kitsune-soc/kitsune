use crate::Compat;

impl Compat for http1::HeaderName {
    type Output = http02::HeaderName;

    fn compat(self) -> Self::Output {
        unsafe { http02::HeaderName::from_bytes(self.as_str().as_bytes()).unwrap_unchecked() }
    }
}

impl Compat for http1::HeaderValue {
    type Output = http02::HeaderValue;

    fn compat(self) -> Self::Output {
        unsafe { http02::HeaderValue::from_maybe_shared_unchecked(self) }
    }
}

impl Compat for http1::HeaderMap {
    type Output = http02::HeaderMap;

    fn compat(self) -> Self::Output {
        self.iter()
            .map(|(name, value)| (name.clone().compat(), value.clone().compat()))
            .collect()
    }
}

impl<B> Compat for http1::Response<B> {
    type Output = http02::Response<B>;

    fn compat(self) -> Self::Output {
        let (parts, body) = self.into_parts();
        let mut res_builder = http02::Response::builder()
            .status(parts.status.compat())
            .version(parts.version.compat());
        *res_builder.headers_mut().unwrap() = parts.headers.compat();

        res_builder.body(body).unwrap()
    }
}

impl Compat for http1::StatusCode {
    type Output = http02::StatusCode;

    fn compat(self) -> Self::Output {
        unsafe { http02::StatusCode::from_u16(self.as_u16()).unwrap_unchecked() }
    }
}

impl Compat for http1::Version {
    type Output = http02::Version;

    fn compat(self) -> Self::Output {
        match self {
            http1::Version::HTTP_09 => http02::Version::HTTP_09,
            http1::Version::HTTP_10 => http02::Version::HTTP_10,
            http1::Version::HTTP_11 => http02::Version::HTTP_11,
            http1::Version::HTTP_2 => http02::Version::HTTP_2,
            http1::Version::HTTP_3 => http02::Version::HTTP_3,
            _ => unreachable!(),
        }
    }
}
