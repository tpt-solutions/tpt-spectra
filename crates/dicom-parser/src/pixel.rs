//! Pixel data extraction from a parsed dataset.
//!
//! For uncompressed transfer syntaxes the pixel data element value is returned
//! as a raw byte buffer (zero-copy reference into the source where possible).
//! Compressed transfer syntaxes are stubbed via `UnsupportedCompression`.

use crate::error::{DicomError, Result};
use crate::model::{Dataset, Tag};
use crate::transfer_syntax::{Compression, TransferSyntax};

/// Raw extracted pixel data.
#[derive(Debug, Clone)]
pub struct PixelData {
    pub rows: u32,
    pub columns: u32,
    pub samples_per_pixel: u32,
    pub bits_allocated: u16,
    pub bits_stored: u16,
    pub pixel_representation: u16,
    /// Raw pixel bytes. Interpretation depends on `bits_allocated`.
    pub data: Vec<u8>,
}

impl PixelData {
    /// Total number of pixels (rows * columns * samples).
    pub fn pixel_count(&self) -> usize {
        (self.rows as usize) * (self.columns as usize) * (self.samples_per_pixel as usize)
    }

    /// Number of bytes per sample given `bits_allocated`.
    pub fn bytes_per_sample(&self) -> usize {
        (self.bits_allocated as usize).div_ceil(8)
    }
}

/// Extract pixel data and geometry from a dataset under the given transfer syntax.
pub fn extract_pixels(ds: &Dataset, ts: TransferSyntax) -> Result<PixelData> {
    match ts.compression {
        Compression::None => {}
        other => {
            return Err(DicomError::UnsupportedCompression(format!("{:?}", other)));
        }
    }

    let pixel = ds
        .get(Tag::PIXEL_DATA)
        .ok_or_else(|| DicomError::MalformedElement {
            tag: Tag::PIXEL_DATA.to_string(),
            len: 0,
        })?;

    let rows = ds.get(Tag::ROWS).and_then(|e| e.as_u16()).unwrap_or(0) as u32;
    let columns = ds.get(Tag::COLUMNS).and_then(|e| e.as_u16()).unwrap_or(0) as u32;
    let samples = ds
        .get(Tag::SAMPLES_PER_PIXEL)
        .and_then(|e| e.as_u16())
        .unwrap_or(1) as u32;
    let bits_allocated = ds
        .get(Tag::BITS_ALLOCATED)
        .and_then(|e| e.as_u16())
        .unwrap_or(16);
    let bits_stored = ds
        .get(Tag::BITS_STORED)
        .and_then(|e| e.as_u16())
        .unwrap_or(bits_allocated);
    let pixel_representation = ds
        .get(Tag::PIXEL_REPRESENTATION)
        .and_then(|e| e.as_u16())
        .unwrap_or(0);

    Ok(PixelData {
        rows,
        columns,
        samples_per_pixel: samples,
        bits_allocated,
        bits_stored,
        pixel_representation,
        data: pixel.value.clone(),
    })
}
