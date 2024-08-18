use crate::{Manifest, SECTION_NAME};
use std::borrow::Cow;
use wasm_encoder::{ComponentSection, CustomSection};

/// Encode a manifest into its proper WASM custom section representation.
///
/// The manifest is encoded in canonical JSON.
/// The emitted byte vector can directly be appended to a WASM component
pub fn encode(manifest: &Manifest<'_>) -> Result<Vec<u8>, sonic_rs::Error> {
    let canonical_manifest = crate::serialise(manifest)?;
    let custom_section = CustomSection {
        name: Cow::Borrowed(SECTION_NAME),
        data: Cow::Owned(canonical_manifest),
    };

    let mut buf = Vec::new();
    custom_section.append_to_component(&mut buf);

    Ok(buf)
}
