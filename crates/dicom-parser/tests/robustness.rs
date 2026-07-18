//! Parser robustness: the parser must never panic on malformed / truncated /
//! random input — it should return an `Err` instead.

use spectra_dicom_parser::parse_file;

#[test]
fn never_panics_on_truncated_input() {
    for len in 0..256usize {
        let mut buf = Vec::with_capacity(len);
        // Deterministic pseudo-random bytes so failures are reproducible.
        let mut state: u32 = (len as u32).wrapping_mul(2654435761);
        for _ in 0..len {
            state = state.wrapping_mul(1664525).wrapping_add(1013904223);
            buf.push((state >> 24) as u8);
        }
        let _ = parse_file(&buf);
    }
}

#[test]
fn never_panics_on_known_bad_prefixes() {
    let cases: Vec<Vec<u8>> = vec![
        vec![],
        vec![0u8; 132],   // no DICM magic
        b"DICM".to_vec(), // magic but no preamble
        {
            let mut v = vec![0u8; 132];
            v.extend_from_slice(b"DICM");
            v
        },
        {
            // DICM + a truncated meta group length element.
            let mut v = vec![0u8; 128];
            v.extend_from_slice(b"DICM");
            v.extend_from_slice(&[0x02, 0x00, 0x00, 0x00, b'U', b'L', 0x04, 0x00]);
            v.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
            v
        },
    ];
    for buf in cases {
        let _ = parse_file(&buf);
    }
}

#[test]
fn empty_buffer_is_error_not_panic() {
    assert!(parse_file(&[]).is_err());
}
