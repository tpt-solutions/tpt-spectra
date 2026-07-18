//! Demo: generate a synthetic CT phantom DICOM, write it to disk, and run the
//! reconstruction pipeline.
//!
//! Run with: `cargo run -p spectra-cli --example phantom -- <out.dcm>`

use spectra_dicom_parser::model::{Dataset, Element, Tag, VR};
use spectra_dicom_parser::writer::{us_element, write_explicit_le};
use spectra_recon_core::geometry::{CtGeometry, Projection};
use spectra_recon_core::modality::{reconstruct_ct, CtFilter, CtParams};
use spectra_recon_core::Volume;
use std::path::PathBuf;

fn main() {
    let out = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("phantom.dcm"));

    let size = 64u16;
    let rows = size as usize;
    let cols = size as usize;

    // Build a disk phantom as a single-slice 16-bit image.
    let mut pixel = vec![0u8; rows * cols * 2];
    let cx = (cols as f32 - 1.0) / 2.0;
    let cy = (rows as f32 - 1.0) / 2.0;
    let r = (size as f32 / 4.0).max(1.0);
    for y in 0..rows {
        for x in 0..cols {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let v = if (dx * dx + dy * dy).sqrt() <= r {
                3000u16
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

    let bytes = write_explicit_le(&meta, &ds);
    std::fs::write(&out, &bytes).expect("write dicom");
    println!("wrote phantom DICOM to {}", out.display());

    // Reconstruct: treat the slice as a single CT projection view.
    let file = spectra_dicom_parser::parse_file(&bytes).expect("parse");
    let px = spectra_dicom_parser::extract_pixels(&file.dataset, file.transfer_syntax).unwrap();
    let mut vol = Volume::new([cols, rows, 1], [1.0, 1.0, 1.0]);
    for y in 0..rows {
        for x in 0..cols {
            let off = (y * cols + x) * 2;
            let v = u16::from_le_bytes([px.data[off], px.data[off + 1]]) as f32;
            vol.set(x, y, 0, 0, v);
        }
    }

    let proj = Projection::new(rows, cols, vol.data.clone());
    let geo = CtGeometry {
        num_views: 1,
        detector: [rows, cols],
        sid: 500.0,
        sdd: 1000.0,
        pixel_size: [1.0, 1.0],
        fov: cols as f32,
    };
    let params = CtParams {
        geometry: geo,
        filter: CtFilter::RamLak,
    };
    let recon = reconstruct_ct(&[proj], &params).expect("reconstruct");
    println!(
        "reconstructed volume {}x{}x{}",
        recon.dims[0], recon.dims[1], recon.dims[2]
    );
}
