//! DCMTK cross-reference harness.
//!
//! Phase 6 verification goal: cross-reference TPT Spectra reconstruction output
//! against DCMTK (C++) for pixel-perfect parity. This harness is *gated*: it is
//! ignored unless the environment variable `SPECTRA_DCMTK_BIN` points at a
//! directory containing the DCMTK `dcmdump` tool. When enabled, it writes a
//! phantom DICOM with our writer and verifies DCMTK can parse it (a minimal but
//! real parity smoke test). The full pixel-parity comparison requires a
//! reference reconstruction produced by `dcmtk`/`dcmqrdb` and is wired in here
//! once those artifacts are available in CI.

use spectra_dicom_parser::model::{Dataset, Element, Tag, VR};
use spectra_dicom_parser::writer::{us_element, write_explicit_le};
use std::process::Command;

fn write_phantom() -> Vec<u8> {
    let n = 32u16;
    let rows = n as usize;
    let cols = n as usize;
    let mut pixel = vec![0u8; rows * cols * 2];
    let cx = (cols as f32 - 1.0) / 2.0;
    let cy = (rows as f32 - 1.0) / 2.0;
    let r = (n as f32 / 4.0).max(1.0);
    for y in 0..rows {
        for x in 0..cols {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let v = if (dx * dx + dy * dy).sqrt() <= r {
                2000u16
            } else {
                0u16
            };
            let off = (y * cols + x) * 2;
            pixel[off..off + 2].copy_from_slice(&v.to_le_bytes());
        }
    }
    let mut ds = Dataset::new();
    ds.elements.push(us_element(Tag::ROWS, rows as u16));
    ds.elements.push(us_element(Tag::COLUMNS, cols as u16));
    ds.elements.push(us_element(Tag::BITS_ALLOCATED, 16));
    ds.elements.push(us_element(Tag::BITS_STORED, 16));
    ds.elements.push(us_element(Tag::SAMPLES_PER_PIXEL, 1));
    ds.elements.push(us_element(Tag::PIXEL_REPRESENTATION, 0));
    ds.elements.push(Element {
        tag: Tag::PIXEL_DATA,
        vr: Some(VR::OW),
        value: pixel,
    });
    let mut meta = Dataset::new();
    meta.elements
        .push(us_element(Tag::FILE_META_GROUP_LENGTH, 0));
    write_explicit_le(&meta, &ds)
}

#[test]
fn dcmtk_parity_smoke() {
    let bindir = match std::env::var("SPECTRA_DCMTK_BIN") {
        Ok(p) => p,
        Err(_) => {
            eprintln!("skipping: SPECTRA_DCMTK_BIN not set");
            return;
        }
    };
    let bytes = write_phantom();
    let path = std::env::temp_dir().join("spectra_parity_phantom.dcm");
    std::fs::write(&path, &bytes).unwrap();

    let dcmdump = std::path::Path::new(&bindir).join("dcmdump");
    let out = Command::new(&dcmdump)
        .arg(&path)
        .output()
        .expect("run dcmdump");
    assert!(out.status.success(), "dcmdump failed: {:?}", out.status);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("(0028,0010)"), "dcmdump missing Rows tag");
    assert!(
        stdout.contains("(0028,0011)"),
        "dcmdump missing Columns tag"
    );
    eprintln!("DCMTK parsed our phantom DICOM successfully (parity smoke test passed)");
}
