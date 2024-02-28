use crate::{Manifest, SECTION_NAME};
use std::borrow::Cow;
use wasm_encoder::{ComponentSection, CustomSection};

pub fn encode(manifest: &Manifest<'_>) -> Result<Vec<u8>, serde_json::Error> {
    let canonical_manifest = crate::serialise(manifest)?;
    let custom_section = CustomSection {
        name: Cow::Borrowed(SECTION_NAME),
        data: Cow::Owned(canonical_manifest),
    };

    let mut buf = Vec::new();
    custom_section.append_to_component(&mut buf);

    Ok(buf)
}
