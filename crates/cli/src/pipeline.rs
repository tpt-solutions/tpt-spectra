//! End-to-end pipeline: DICOM parsing -> reconstruction -> AI diagnostics.

use anyhow::{Context, Result};
use spectra_ai_bridge::{annotations, parse_scores, DiagnosticModel, ModelRegistry};
use spectra_dicom_parser as dicom;
use spectra_recon_core::geometry::{CtGeometry, Projection};
use spectra_recon_core::modality::{reconstruct_ct, CtFilter, CtParams};
use spectra_recon_core::Volume;

/// Parse a DICOM file from disk into a [`dicom::DicomFile`].
pub fn load_dicom(path: &std::path::Path) -> Result<dicom::DicomFile> {
    let bytes = std::fs::read(path).with_context(|| format!("reading {}", path.display()))?;
    dicom::parse_file(&bytes).context("parsing DICOM file")
}

/// Convert parsed pixel data into a scalar [`Volume`] (single 2D slice or stack
/// of slices from a multi-frame image).
pub fn pixels_to_volume(px: &dicom::PixelData) -> Volume {
    let rows = px.rows.max(1) as usize;
    let cols = px.columns.max(1) as usize;
    let frames = px.data.len() / (rows * cols * (px.bits_allocated as usize).div_ceil(8)).max(1);
    let frames = frames.max(1);
    let mut vol = Volume::new([cols, rows, frames], [1.0, 1.0, 1.0]);
    let bytes_per_sample = (px.bits_allocated as usize).div_ceil(8);
    for f in 0..frames {
        for y in 0..rows {
            for x in 0..cols {
                let off = (f * rows * cols + y * cols + x) * bytes_per_sample;
                let v = match bytes_per_sample {
                    1 => px.data[off] as f32,
                    2 => {
                        let raw = u16::from_le_bytes([px.data[off], px.data[off + 1]]);
                        raw as f32
                    }
                    _ => 0.0,
                };
                vol.set(x, y, f, 0, v);
            }
        }
    }
    vol
}

/// Treat each 2D slice of a volume as a CT projection view and reconstruct.
pub fn reconstruct_from_volume(vol: &Volume) -> Result<Volume> {
    if vol.dims[2] < 1 {
        anyhow::bail!("volume has no slices to use as projections");
    }
    let det_cols = vol.dims[0];
    let det_rows = vol.dims[1];
    let views = vol.dims[2];
    let mut projections = Vec::with_capacity(views);
    for z in 0..views {
        let mut data = vec![0.0f32; det_rows * det_cols];
        for y in 0..det_rows {
            for x in 0..det_cols {
                data[y * det_cols + x] = vol.get(x, y, z, 0);
            }
        }
        projections.push(Projection::new(det_rows, det_cols, data));
    }
    let geo = CtGeometry {
        num_views: views,
        detector: [det_rows, det_cols],
        sid: 500.0,
        sdd: 1000.0,
        pixel_size: [1.0, 1.0],
        fov: det_cols as f32,
    };
    let params = CtParams {
        geometry: geo,
        filter: CtFilter::RamLak,
    };
    reconstruct_ct(&projections, &params).context("CT reconstruction")
}

/// Run an example diagnostic model over a reconstructed volume (requires the
/// ONNX model file referenced by the registry to exist on disk).
pub fn run_diagnostic(volume: &Volume, model_id: &str) -> Result<Vec<String>> {
    let registry = ModelRegistry::with_examples();
    let desc = registry
        .get(model_id)
        .with_context(|| format!("unknown model id {model_id}"))?
        .clone();
    let mut model = DiagnosticModel::load(&desc).context("loading model")?;
    let out = model.infer(volume).context("inference")?;
    let scores: Vec<f32> = out.iter().copied().collect();
    let diag = parse_scores(&scores, &[]);
    Ok(annotations(&diag, 0.5))
}
