use crate::Manifest;
use miette::Diagnostic;
use thiserror::Error;
use wasmparser::Payload;

const SECTION_NAME: &str = "manifest-v0";

#[derive(Debug, Diagnostic, Error)]
pub enum ParseError {
    #[error(transparent)]
    Parse(#[from] serde_json::Error),

    #[error(transparent)]
    WarmParse(#[from] wasmparser::BinaryReaderError),
}

impl<'a> Manifest<'a> {
    pub fn parse(module: &'a [u8]) -> Result<Option<Self>, ParseError> {
        let mut sections = wasmparser::Parser::new(0).parse_all(module);
        let data = loop {
            match sections.next().transpose()? {
                Some(Payload::CustomSection(reader)) if reader.name() == SECTION_NAME => {
                    break reader.data();
                }
                Some(..) => {
                    // Section we don't care about. Skip.
                }
                None => return Ok(None),
            }
        };

        Ok(Some(serde_json::from_slice(data)?))
    }
}
