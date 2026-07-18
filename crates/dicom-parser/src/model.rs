//! Core DICOM data model: tags, value representations (VR), elements, datasets.

/// A DICOM tag, identified by a 4-byte group/element pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Tag {
    pub group: u16,
    pub element: u16,
}

impl Tag {
    pub const fn new(group: u16, element: u16) -> Self {
        Self { group, element }
    }

    /// File meta information group length (0002,0000).
    pub const FILE_META_GROUP_LENGTH: Tag = Tag::new(0x0002, 0x0000);
    /// Media storage SOP class UID (0002,0002).
    pub const MEDIA_STORAGE_SOP_CLASS_UID: Tag = Tag::new(0x0002, 0x0002);
    /// Media storage SOP instance UID (0002,0003).
    pub const MEDIA_STORAGE_SOP_INSTANCE_UID: Tag = Tag::new(0x0002, 0x0003);
    /// Transfer syntax UID (0002,0010).
    pub const TRANSFER_SYNTAX_UID: Tag = Tag::new(0x0002, 0x0010);
    /// Pixel data (7FE0,0010).
    pub const PIXEL_DATA: Tag = Tag::new(0x7FE0, 0x0010);
    /// Rows (0028,0010).
    pub const ROWS: Tag = Tag::new(0x0028, 0x0010);
    /// Columns (0028,0011).
    pub const COLUMNS: Tag = Tag::new(0x0028, 0x0011);
    /// Bits allocated (0028,0100).
    pub const BITS_ALLOCATED: Tag = Tag::new(0x0028, 0x0100);
    /// Bits stored (0028,0101).
    pub const BITS_STORED: Tag = Tag::new(0x0028, 0x0101);
    /// Pixel representation (0028,0103).
    pub const PIXEL_REPRESENTATION: Tag = Tag::new(0x0028, 0x0103);
    /// Samples per pixel (0028,0002).
    pub const SAMPLES_PER_PIXEL: Tag = Tag::new(0x0028, 0x0002);
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:04X},{:04X})", self.group, self.element)
    }
}

/// DICOM Value Representations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum VR {
    AE,
    AS,
    AT,
    CS,
    DA,
    DS,
    DT,
    FL,
    FD,
    IS,
    LO,
    LT,
    OB,
    OD,
    OF,
    OL,
    OW,
    PN,
    SH,
    SL,
    SQ,
    SS,
    ST,
    TM,
    UI,
    UL,
    UN,
    US,
    UT,
}

impl VR {
    /// VRs with a 2-byte explicit length header (all except OB/OW/OF/SQ/UT/UN
    /// with undefined length, which use 4-byte length in explicit VR).
    pub fn has_short_form(self) -> bool {
        !matches!(
            self,
            VR::OB | VR::OW | VR::OF | VR::OD | VR::OL | VR::SQ | VR::UT | VR::UN
        )
    }

    /// Returns true if this VR is a string type (with padding rules).
    pub fn is_string(self) -> bool {
        matches!(
            self,
            VR::AE
                | VR::AS
                | VR::CS
                | VR::DA
                | VR::DS
                | VR::DT
                | VR::IS
                | VR::LO
                | VR::LT
                | VR::PN
                | VR::SH
                | VR::ST
                | VR::TM
                | VR::UI
                | VR::UT
        )
    }

    /// Encode this VR as its 2-byte explicit code (e.g. `OB` -> [b'O', b'B']).
    #[allow(clippy::byte_char_slices)]
    pub fn code(self) -> [u8; 2] {
        match self {
            VR::AE => [b'A', b'E'],
            VR::AS => [b'A', b'S'],
            VR::AT => [b'A', b'T'],
            VR::CS => [b'C', b'S'],
            VR::DA => [b'D', b'A'],
            VR::DS => [b'D', b'S'],
            VR::DT => [b'D', b'T'],
            VR::FL => [b'F', b'L'],
            VR::FD => [b'F', b'D'],
            VR::IS => [b'I', b'S'],
            VR::LO => [b'L', b'O'],
            VR::LT => [b'L', b'T'],
            VR::OB => [b'O', b'B'],
            VR::OD => [b'O', b'D'],
            VR::OF => [b'O', b'F'],
            VR::OL => [b'O', b'L'],
            VR::OW => [b'O', b'W'],
            VR::PN => [b'P', b'N'],
            VR::SH => [b'S', b'H'],
            VR::SL => [b'S', b'L'],
            VR::SQ => [b'S', b'Q'],
            VR::SS => [b'S', b'S'],
            VR::ST => [b'S', b'T'],
            VR::TM => [b'T', b'M'],
            VR::UI => [b'U', b'I'],
            VR::UL => [b'U', b'L'],
            VR::UN => [b'U', b'N'],
            VR::US => [b'U', b'S'],
            VR::UT => [b'U', b'T'],
        }
    }
}

/// Map a 2-byte explicit VR code to `VR`. Returns `None` if unknown.
#[allow(clippy::byte_char_slices)]
pub fn vr_from_code(code: [u8; 2]) -> Option<VR> {
    Some(match code {
        [b'A', b'E'] => VR::AE,
        [b'A', b'S'] => VR::AS,
        [b'A', b'T'] => VR::AT,
        [b'C', b'S'] => VR::CS,
        [b'D', b'A'] => VR::DA,
        [b'D', b'S'] => VR::DS,
        [b'D', b'T'] => VR::DT,
        [b'F', b'L'] => VR::FL,
        [b'F', b'D'] => VR::FD,
        [b'I', b'S'] => VR::IS,
        [b'L', b'O'] => VR::LO,
        [b'L', b'T'] => VR::LT,
        [b'O', b'B'] => VR::OB,
        [b'O', b'D'] => VR::OD,
        [b'O', b'F'] => VR::OF,
        [b'O', b'L'] => VR::OL,
        [b'O', b'W'] => VR::OW,
        [b'P', b'N'] => VR::PN,
        [b'S', b'H'] => VR::SH,
        [b'S', b'L'] => VR::SL,
        [b'S', b'Q'] => VR::SQ,
        [b'S', b'S'] => VR::SS,
        [b'S', b'T'] => VR::ST,
        [b'T', b'M'] => VR::TM,
        [b'U', b'I'] => VR::UI,
        [b'U', b'L'] => VR::UL,
        [b'U', b'N'] => VR::UN,
        [b'U', b'S'] => VR::US,
        [b'U', b'T'] => VR::UT,
        _ => return None,
    })
}

/// A single dataset element.
#[derive(Debug, Clone)]
pub struct Element {
    pub tag: Tag,
    pub vr: Option<VR>,
    /// Raw value bytes. For pixel data this may reference the source buffer.
    pub value: Vec<u8>,
}

impl Element {
    /// Interpret value as a string (UTF-8 lossy), trimming DICOM padding.
    pub fn as_str(&self) -> String {
        let mut s = String::from_utf8_lossy(&self.value).into_owned();
        while s.ends_with(' ') || s.ends_with('\0') {
            s.pop();
        }
        s
    }

    /// Interpret value as an unsigned 16-bit integer (little endian).
    pub fn as_u16(&self) -> Option<u16> {
        self.value
            .get(0..2)
            .map(|b| u16::from_le_bytes([b[0], b[1]]))
    }

    /// Interpret value as an unsigned 32-bit integer (little endian).
    pub fn as_u32(&self) -> Option<u32> {
        self.value
            .get(0..4)
            .map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
}

/// A parsed DICOM dataset: an ordered collection of elements.
#[derive(Debug, Clone, Default)]
pub struct Dataset {
    pub elements: Vec<Element>,
}

impl Dataset {
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up the first element with the given tag.
    pub fn get(&self, tag: Tag) -> Option<&Element> {
        self.elements.iter().find(|e| e.tag == tag)
    }

    /// Get the string value of a tag, if present.
    pub fn get_str(&self, tag: Tag) -> Option<String> {
        self.get(tag).map(|e| e.as_str())
    }
}
