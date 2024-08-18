use crate::Manifest;
use fast_cjson::CanonicalFormatter;
use serde::Serialize;

/// Serialise a manifest into its canonical JSON representation
pub fn serialise(manifest: &Manifest<'_>) -> Result<Vec<u8>, sonic_rs::Error> {
    let mut buf = Vec::new();
    let mut ser = sonic_rs::Serializer::with_formatter(&mut buf, CanonicalFormatter::new());
    manifest.serialize(&mut ser)?;

    Ok(buf)
}
