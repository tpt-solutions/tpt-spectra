//! Minimal DICOM writer for generating test fixtures and phantom images.
//!
//! Serializes datasets in Explicit VR Little Endian (uncompressed). This is
//! intentionally small — enough to produce valid files for the parser and for
//! phantom-based verification, not a full DICOM writer.

use crate::model::{Dataset, Element, Tag, VR};
use crate::transfer_syntax::{ByteOrder, VREncoding, EXPLICIT_VR_LE};

/// Serialize a dataset to Explicit VR Little Endian bytes (with preamble + DICM).
///
/// `meta` is accepted for API symmetry; the file meta group is always written
/// with the Explicit VR LE transfer syntax UID.
pub fn write_explicit_le(_meta: &Dataset, dataset: &Dataset) -> Vec<u8> {
    let mut buf = vec![0u8; 128];
    buf.extend_from_slice(b"DICM");

    // File meta: transfer syntax UID = Explicit VR LE.
    let ts_uid = EXPLICIT_VR_LE.uid;
    let mut ts_elem: Vec<u8> = vec![0x02, 0x00, 0x10, 0x00, b'U', b'I'];
    ts_elem.extend_from_slice(&(ts_uid.len() as u16).to_le_bytes());
    ts_elem.extend_from_slice(ts_uid.as_bytes());

    let group_len = ts_elem.len() as u32;
    buf.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, b'U', b'L', 0x04, 0x00]);
    buf.extend_from_slice(&group_len.to_le_bytes());
    buf.extend_from_slice(&ts_elem);

    write_dataset(
        &mut buf,
        dataset,
        ByteOrder::LittleEndian,
        VREncoding::Explicit,
    );
    buf
}

/// Serialize a dataset to Implicit VR Little Endian bytes (no preamble/meta).
pub fn write_implicit_le(dataset: &Dataset) -> Vec<u8> {
    let mut buf = Vec::new();
    write_dataset(
        &mut buf,
        dataset,
        ByteOrder::LittleEndian,
        VREncoding::Implicit,
    );
    buf
}

fn write_dataset(buf: &mut Vec<u8>, ds: &Dataset, _order: ByteOrder, enc: VREncoding) {
    for e in &ds.elements {
        let vr = e.vr.unwrap_or(VR::UN);
        if enc == VREncoding::Implicit {
            buf.extend_from_slice(&e.tag.group.to_le_bytes());
            buf.extend_from_slice(&e.tag.element.to_le_bytes());
            buf.extend_from_slice(&(e.value.len() as u32).to_le_bytes());
            buf.extend_from_slice(&e.value);
            continue;
        }
        buf.extend_from_slice(&e.tag.group.to_le_bytes());
        buf.extend_from_slice(&e.tag.element.to_le_bytes());
        buf.extend_from_slice(&vr.code());
        if vr.has_short_form() {
            buf.extend_from_slice(&(e.value.len() as u16).to_le_bytes());
            buf.extend_from_slice(&e.value);
        } else {
            buf.extend_from_slice(&[0, 0]);
            buf.extend_from_slice(&(e.value.len() as u32).to_le_bytes());
            buf.extend_from_slice(&e.value);
        }
    }
}

/// Convenience: build a single US (unsigned short) element.
pub fn us_element(tag: Tag, value: u16) -> Element {
    Element {
        tag,
        vr: Some(VR::US),
        value: value.to_le_bytes().to_vec(),
    }
}
