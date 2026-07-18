//! Transfer syntax handling: implicit/explicit VR, endianness, compression.

use crate::model::VR;
use std::fmt;

/// A DICOM transfer syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransferSyntax {
    pub uid: &'static str,
    pub name: &'static str,
    pub byte_order: ByteOrder,
    pub vr_encoding: VREncoding,
    pub compression: Compression,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteOrder {
    LittleEndian,
    BigEndian,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VREncoding {
    Explicit,
    Implicit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compression {
    None,
    Jpeg,
    Jpeg2000,
    Rle,
    /// Unknown / unsupported compression — stub for later.
    Unsupported,
}

impl fmt::Display for TransferSyntax {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.uid)
    }
}

/// Implicit VR Little Endian (the only mandatory transfer syntax).
pub const IMPLICIT_VR_LE: TransferSyntax = TransferSyntax {
    uid: "1.2.840.10008.1.2",
    name: "Implicit VR Little Endian",
    byte_order: ByteOrder::LittleEndian,
    vr_encoding: VREncoding::Implicit,
    compression: Compression::None,
};

/// Explicit VR Little Endian.
pub const EXPLICIT_VR_LE: TransferSyntax = TransferSyntax {
    uid: "1.2.840.10008.1.2.1",
    name: "Explicit VR Little Endian",
    byte_order: ByteOrder::LittleEndian,
    vr_encoding: VREncoding::Explicit,
    compression: Compression::None,
};

/// Explicit VR Big Endian (retired).
pub const EXPLICIT_VR_BE: TransferSyntax = TransferSyntax {
    uid: "1.2.840.10008.1.2.2",
    name: "Explicit VR Big Endian",
    byte_order: ByteOrder::BigEndian,
    vr_encoding: VREncoding::Explicit,
    compression: Compression::None,
};

/// JPEG Baseline (lossy).
pub const JPEG_BASELINE: TransferSyntax = TransferSyntax {
    uid: "1.2.840.10008.1.2.4.50",
    name: "JPEG Baseline",
    byte_order: ByteOrder::LittleEndian,
    vr_encoding: VREncoding::Explicit,
    compression: Compression::Jpeg,
};

/// JPEG 2000.
pub const JPEG_2000: TransferSyntax = TransferSyntax {
    uid: "1.2.840.10008.1.2.4.90",
    name: "JPEG 2000 (Lossless)",
    byte_order: ByteOrder::LittleEndian,
    vr_encoding: VREncoding::Explicit,
    compression: Compression::Jpeg2000,
};

/// RLE Lossless.
pub const RLE_LOSSLESS: TransferSyntax = TransferSyntax {
    uid: "1.2.840.10008.1.2.5",
    name: "RLE Lossless",
    byte_order: ByteOrder::LittleEndian,
    vr_encoding: VREncoding::Explicit,
    compression: Compression::Rle,
};

/// All known transfer syntaxes.
pub const KNOWN: &[TransferSyntax] = &[
    IMPLICIT_VR_LE,
    EXPLICIT_VR_LE,
    EXPLICIT_VR_BE,
    JPEG_BASELINE,
    JPEG_2000,
    RLE_LOSSLESS,
];

/// Resolve a transfer syntax UID string to a known syntax.
///
/// Strips trailing null/space padding per DICOM UID rules. Unknown UIDs are
/// reported as unsupported compression (stub for later implementation).
pub fn lookup(uid: &str) -> TransferSyntax {
    let uid = uid.trim_end_matches('\0').trim_end();
    for ts in KNOWN {
        if ts.uid == uid {
            return *ts;
        }
    }
    // Unknown — assume explicit VR LE with unsupported compression for now.
    TransferSyntax {
        uid: "1.2.840.10008.1.2.1",
        name: "Unknown",
        byte_order: ByteOrder::LittleEndian,
        vr_encoding: VREncoding::Explicit,
        compression: Compression::Unsupported,
    }
}

/// Helper: read a u16 in the given byte order.
pub fn read_u16(buf: &[u8], order: ByteOrder) -> u16 {
    let b = [buf[0], buf[1]];
    match order {
        ByteOrder::LittleEndian => u16::from_le_bytes(b),
        ByteOrder::BigEndian => u16::from_be_bytes(b),
    }
}

/// Helper: read a u32 in the given byte order.
pub fn read_u32(buf: &[u8], order: ByteOrder) -> u32 {
    let b = [buf[0], buf[1], buf[2], buf[3]];
    match order {
        ByteOrder::LittleEndian => u32::from_le_bytes(b),
        ByteOrder::BigEndian => u32::from_be_bytes(b),
    }
}

/// Helper: VR encoding with explicit VR code (used in explicit datasets).
pub fn _explicit_vr() -> VREncoding {
    VREncoding::Explicit
}

#[allow(dead_code)]
fn _assert_vr(_: VR) {}
