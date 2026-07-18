//! Error types for the DICOM parser.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DicomError {
    #[error("unexpected end of input: needed {needed} bytes, had {available}")]
    UnexpectedEof { needed: usize, available: usize },

    #[error("invalid preamble: expected DICM magic at offset 128")]
    MissingMagic,

    #[error("invalid file meta group length: {0}")]
    BadGroupLength(usize),

    #[error("unknown explicit VR code: {0:?}")]
    UnknownVR([u8; 2]),

    #[error("unsupported transfer syntax: {0}")]
    UnsupportedTransferSyntax(String),

    #[error("unsupported compressed pixel data: {0}")]
    UnsupportedCompression(String),

    #[error("malformed element: tag {tag} has invalid length {len}")]
    MalformedElement { tag: String, len: u32 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DicomError>;
