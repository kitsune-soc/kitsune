use super::SignatureHeader;
use std::fmt::Write;

#[inline]
pub fn serialise<'a, I>(header: SignatureHeader<'_, I, &str>) -> String
where
    I: Iterator<Item = &'a str>,
{
    let mut buffer = String::new();

    let _ = write!(buffer, "keyId=\"{}\"", header.key_id);

    buffer.push_str("headers=\"");
    for item in itertools::intersperse(header.headers, " ") {
        buffer.push_str(item);
    }
    buffer.push('"');

    let _ = write!(buffer, ",signature=\"{}\"", header.signature);

    if let Some(created) = header.created {
        let _ = write!(buffer, ",created={created}");
    }

    if let Some(expires) = header.expires {
        let _ = write!(buffer, ",expires={expires}");
    }

    buffer
}
