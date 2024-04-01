use crate::{Manifest, SECTION_NAME};
use std::{io, ops::Range};
use thiserror::Error;
use wasmparser::Payload;

/// Type specifying the range of a section
pub type SectionRange = Range<usize>;

/// Error while decoding the manifest from a WASM module
#[derive(Debug, Error)]
pub enum DecodeError {
    /// Parsing of the JSON manifest failed
    #[error(transparent)]
    Parse(#[from] serde_json::Error),

    /// Parsing of the WASM component failed
    #[error(transparent)]
    WarmParse(#[from] wasmparser::BinaryReaderError),
}

/// Decode a manifest from a WASM module
///
/// If it was found a tuple consisting of the manifest and the custom section (including its type ID and length) is returned.
pub fn decode(module: &[u8]) -> Result<Option<(Manifest<'_>, SectionRange)>, DecodeError> {
    let mut sections = wasmparser::Parser::new(0).parse_all(module);
    let payload = loop {
        match sections.next().transpose()? {
            Some(Payload::CustomSection(reader)) if reader.name() == SECTION_NAME => {
                break reader;
            }
            Some(..) => {
                // Section we don't care about. Skip.
            }
            None => return Ok(None),
        }
    };

    // Check the size of the LEB128 encoded integer
    let length_size =
        leb128::write::unsigned(&mut io::sink(), payload.data().len() as u64).unwrap();
    let start_offset = 1 + length_size; // 1 byte for the section identifier, N bytes for the length of the section

    let mut section_range = payload.range();
    section_range.start -= start_offset;

    let manifest = serde_json::from_slice(payload.data())?;

    Ok(Some((manifest, section_range)))
}
