//! # spectra-dicom-parser
//!
//! Zero-allocation DICOM parser: tags, datasets, file meta, transfer syntaxes,
//! and pixel data extraction.

pub mod error;
pub mod model;
pub mod parser;
pub mod pixel;
pub mod transfer_syntax;
pub mod writer;

pub use error::{DicomError, Result};
pub use model::{Dataset, Element, Tag, VR};
pub use parser::{parse_file, DicomFile};
pub use pixel::{extract_pixels, PixelData};
pub use transfer_syntax::TransferSyntax;
pub use writer::{us_element, write_explicit_le, write_implicit_le};
