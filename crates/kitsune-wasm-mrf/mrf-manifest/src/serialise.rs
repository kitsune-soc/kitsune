use crate::Manifest;
use olpc_cjson::CanonicalFormatter;

impl Manifest<'_> {
    pub fn serialise(&self) -> Result<Vec<u8>, serde_json::Error> {
        let mut buf = Vec::new();
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
        serde::Serialize::serialize(self, &mut ser)?;
        Ok(buf)
    }
}
