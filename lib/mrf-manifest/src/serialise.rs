use crate::Manifest;
use olpc_cjson::CanonicalFormatter;
use serde::Serialize;

/// Serialise a manifest into its canonical JSON representation
pub fn serialise(manifest: &Manifest<'_>) -> Result<Vec<u8>, serde_json::Error> {
    let mut buf = Vec::new();
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
    manifest.serialize(&mut ser)?;
    Ok(buf)
}
