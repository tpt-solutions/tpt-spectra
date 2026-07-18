//! Main DICOM parser: file meta header + dataset parsing.
//!
//! Supports Implicit/Explicit VR, Little/Big Endian, and uncompressed pixel
//! data extraction. Compressed syntaxes are recognized but deferred (stub).

use crate::error::{DicomError, Result};
use crate::model::{vr_from_code, Dataset, Element, Tag, VR};
use crate::transfer_syntax::{
    lookup, read_u16, read_u32, ByteOrder, TransferSyntax, VREncoding, IMPLICIT_VR_LE,
};

/// A fully parsed DICOM file.
#[derive(Debug, Clone)]
pub struct DicomFile {
    pub preamble: Option<[u8; 128]>,
    pub meta: Dataset,
    pub transfer_syntax: TransferSyntax,
    pub dataset: Dataset,
}

/// Parse a complete DICOM file from raw bytes.
pub fn parse_file(input: &[u8]) -> Result<DicomFile> {
    let mut pos = 0usize;

    // Optional 128-byte preamble + "DICM" magic.
    let preamble = if input.len() >= 132 && &input[128..132] == b"DICM" {
        let mut p = [0u8; 128];
        p.copy_from_slice(&input[0..128]);
        pos = 132;
        Some(p)
    } else {
        None
    };

    // File meta header (always Explicit VR Little Endian, group 0002).
    let (meta, meta_end, group_length) = parse_file_meta(input, pos)?;
    let _ = group_length;
    pos = meta_end;

    let ts_uid = meta
        .get(Tag::TRANSFER_SYNTAX_UID)
        .map(|e| e.as_str())
        .unwrap_or_else(|| IMPLICIT_VR_LE.uid.to_string());
    let transfer_syntax = lookup(&ts_uid);

    let order = transfer_syntax.byte_order;
    let vr_enc = transfer_syntax.vr_encoding;

    let dataset = parse_dataset(input, pos, order, vr_enc, true)?;

    Ok(DicomFile {
        preamble,
        meta,
        transfer_syntax,
        dataset,
    })
}

/// Parse the file meta information group (0002,xxxx), Explicit VR LE.
fn parse_file_meta(input: &[u8], start: usize) -> Result<(Dataset, usize, usize)> {
    let mut pos = start;
    let mut meta = Dataset::new();

    // (0002,0000) File Meta Group Length — first element.
    let (tag, _vr, len, next) =
        read_element_header(input, pos, ByteOrder::LittleEndian, VREncoding::Explicit)?;
    if tag != Tag::FILE_META_GROUP_LENGTH {
        return Err(DicomError::BadGroupLength(pos));
    }
    pos = next;
    let group_length = read_u32(&input[pos..pos + len as usize], ByteOrder::LittleEndian) as usize;
    pos += len as usize;

    let meta_end = pos + group_length;
    while pos < meta_end {
        let (tag, vr, len, next) =
            read_element_header(input, pos, ByteOrder::LittleEndian, VREncoding::Explicit)?;
        pos = next;
        let value = input
            .get(pos..pos + len as usize)
            .ok_or(DicomError::UnexpectedEof {
                needed: len as usize,
                available: input.len() - pos,
            })?
            .to_vec();
        pos += len as usize;
        meta.elements.push(Element { tag, vr, value });
    }

    Ok((meta, meta_end, group_length))
}

/// Parse the main dataset until end of input.
fn parse_dataset(
    input: &[u8],
    start: usize,
    order: ByteOrder,
    vr_enc: VREncoding,
    _top: bool,
) -> Result<Dataset> {
    let mut ds = Dataset::new();
    let mut pos = start;
    while pos < input.len() {
        // Need at least group+element (4 bytes).
        if input.len() - pos < 4 {
            break;
        }
        let (tag, vr, len, next) = match read_element_header(input, pos, order, vr_enc) {
            Ok(h) => h,
            Err(DicomError::UnexpectedEof { .. }) => break,
            Err(e) => return Err(e),
        };
        pos = next;

        let value = if len == 0xFFFFFFFF {
            // Undefined length: stop at matching sequence delimiter. For non-SQ
            // this is an error; we treat as unsupported and bail gracefully.
            if vr != Some(VR::SQ) {
                return Err(DicomError::MalformedElement {
                    tag: tag.to_string(),
                    len,
                });
            }
            // Consume until item/delimiter — stubbed: skip to end.
            let rest = &input[pos..];
            let consumed = find_sequence_delimiter(rest).unwrap_or(rest.len());
            let v = rest[..consumed].to_vec();
            pos += consumed;
            v
        } else {
            let end = pos + len as usize;
            let value = input
                .get(pos..end)
                .ok_or(DicomError::UnexpectedEof {
                    needed: len as usize,
                    available: input.len() - pos,
                })?
                .to_vec();
            pos = end;
            value
        };

        ds.elements.push(Element { tag, vr, value });
    }
    Ok(ds)
}

/// Read an element header, returning (tag, vr, length, pos_after_header).
fn read_element_header(
    input: &[u8],
    pos: usize,
    order: ByteOrder,
    vr_enc: VREncoding,
) -> Result<(Tag, Option<VR>, u32, usize)> {
    let need = 8usize;
    if input.len() < pos + need {
        return Err(DicomError::UnexpectedEof {
            needed: need,
            available: input.len().saturating_sub(pos),
        });
    }
    let group = read_u16(&input[pos..pos + 2], order);
    let element = read_u16(&input[pos + 2..pos + 4], order);
    let tag = Tag::new(group, element);

    if vr_enc == VREncoding::Implicit {
        let len = read_u32(&input[pos + 4..pos + 8], order);
        return Ok((tag, None, len, pos + 8));
    }

    // Explicit VR: 2-byte VR code, then either 2-byte reserved + 4-byte length
    // (short form) or 4-byte length directly (long form).
    let vr_code = [input[pos + 4], input[pos + 5]];
    let vr = vr_from_code(vr_code).ok_or(DicomError::UnknownVR(vr_code))?;

    if vr.has_short_form() {
        // 2-byte length.
        if input.len() < pos + 8 {
            return Err(DicomError::UnexpectedEof {
                needed: 8,
                available: input.len().saturating_sub(pos),
            });
        }
        let len = read_u16(&input[pos + 6..pos + 8], order) as u32;
        Ok((tag, Some(vr), len, pos + 8))
    } else {
        // 4-byte length. Note: long-form length is always little endian in DICOM.
        if input.len() < pos + 12 {
            return Err(DicomError::UnexpectedEof {
                needed: 12,
                available: input.len().saturating_sub(pos),
            });
        }
        let len = read_u32(&input[pos + 8..pos + 12], ByteOrder::LittleEndian);
        Ok((tag, Some(vr), len, pos + 12))
    }
}

/// Find the sequence delimiter item (FFFE,E0DD) within a buffer.
fn find_sequence_delimiter(buf: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i + 4 <= buf.len() {
        if buf[i] == 0xFE && buf[i + 1] == 0xFF && buf[i + 2] == 0xDD && buf[i + 3] == 0xE0 {
            return Some(i);
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Tag;
    use crate::pixel::extract_pixels;

    /// Build a minimal Explicit VR LE DICOM file with a small pixel dataset.
    fn build_sample() -> Vec<u8> {
        let mut buf = vec![0u8; 128]; // preamble
        buf.extend_from_slice(b"DICM");

        // File meta group: (0002,0010) Transfer Syntax UID = Explicit VR LE.
        let ts_uid = b"1.2.840.10008.1.2.1";
        // TS element: (0002,0010) UI, short form, length = uid len.
        let ts_elem: Vec<u8> = {
            let mut v = vec![0x02, 0x00, 0x10, 0x00, b'U', b'I'];
            v.extend_from_slice(&(ts_uid.len() as u16).to_le_bytes());
            v.extend_from_slice(ts_uid);
            v
        };
        // Group length = total bytes of all meta elements after the group-length element.
        let group_len = ts_elem.len() as u32;
        let mut out = buf;
        // (0002,0000) UL group length (value filled below).
        out.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, b'U', b'L', 0x04, 0x00]);
        out.extend_from_slice(&group_len.to_le_bytes());
        out.extend_from_slice(&ts_elem);

        let pixel = [0x11u8, 0x22, 0x33, 0x44];
        let rows = 2u16;
        let cols = 2u16;

        // (0028,0010) US Rows = 2
        out.extend_from_slice(&[0x28, 0x00, 0x10, 0x00, b'U', b'S', 0x02, 0x00]);
        out.extend_from_slice(&rows.to_le_bytes());
        // (0028,0011) US Columns = 2
        out.extend_from_slice(&[0x28, 0x00, 0x11, 0x00, b'U', b'S', 0x02, 0x00]);
        out.extend_from_slice(&cols.to_le_bytes());
        // (0028,0100) US BitsAllocated = 16
        out.extend_from_slice(&[0x28, 0x00, 0x00, 0x01, b'U', b'S', 0x02, 0x00]);
        out.extend_from_slice(&16u16.to_le_bytes());
        // (7FE0,0010) OW PixelData (long form) = 4 bytes
        out.extend_from_slice(&[0xE0, 0x7F, 0x10, 0x00, b'O', b'W', 0x00, 0x00]);
        out.extend_from_slice(&(pixel.len() as u32).to_le_bytes());
        out.extend_from_slice(&pixel);
        out
    }

    #[test]
    fn parse_explicit_vr_le_roundtrip() {
        let bytes = build_sample();
        let file = parse_file(&bytes).expect("parse should succeed");
        assert_eq!(file.transfer_syntax.uid, "1.2.840.10008.1.2.1");
        assert_eq!(file.dataset.get(Tag::ROWS).unwrap().as_u16(), Some(2));
        assert_eq!(file.dataset.get(Tag::COLUMNS).unwrap().as_u16(), Some(2));
        let px = extract_pixels(&file.dataset, file.transfer_syntax).unwrap();
        assert_eq!(px.rows, 2);
        assert_eq!(px.columns, 2);
        assert_eq!(px.data, vec![0x11, 0x22, 0x33, 0x44]);
    }

    #[test]
    fn writer_then_parse_roundtrip() {
        use crate::model::Dataset;
        use crate::writer::{us_element, write_explicit_le};
        let mut ds = Dataset::new();
        ds.elements.push(us_element(Tag::ROWS, 16));
        ds.elements.push(us_element(Tag::COLUMNS, 16));
        let mut meta = Dataset::new();
        meta.elements
            .push(us_element(Tag::FILE_META_GROUP_LENGTH, 0));
        let bytes = write_explicit_le(&meta, &ds);
        let file = parse_file(&bytes).expect("roundtrip parse");
        assert_eq!(file.dataset.get(Tag::ROWS).unwrap().as_u16(), Some(16));
        assert_eq!(file.dataset.get(Tag::COLUMNS).unwrap().as_u16(), Some(16));
    }
}
