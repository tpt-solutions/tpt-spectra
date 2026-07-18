//! Voxel → ONNX tensor adapter: shape and normalize reconstructed volumes.

use crate::registry::NormalizationKind;
use ndarray::ArrayD;
use spectra_recon_core::Volume;

/// Extension to map registry normalization kinds onto adapter normalizations.
pub trait NormalizationKindExt {
    fn to_norm(self) -> Normalization;
}

impl NormalizationKindExt for NormalizationKind {
    fn to_norm(self) -> Normalization {
        match self {
            NormalizationKind::MinMax => Normalization::MinMax,
            NormalizationKind::ZScore => Normalization::ZScore,
            NormalizationKind::None => Normalization::None,
        }
    }
}

/// Normalization strategy applied before feeding a volume to a model.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Normalization {
    /// Scale to [0, 1] using global min/max.
    MinMax,
    /// Zero mean, unit variance.
    ZScore,
    /// No normalization (model handles preprocessing).
    None,
}

/// Build a 5D NCDHW tensor (`[batch=1, channels=1, depth, height, width]`) from
/// a scalar volume, ready for 3D ONNX inference.
pub fn to_nchw(volume: &Volume, norm: Normalization) -> ArrayD<f32> {
    let [w, h, d] = volume.dims;
    let mut flat: Vec<f32> = volume.data.clone();
    apply_normalization(&mut flat, norm);

    let mut out = ArrayD::<f32>::zeros(vec![1, 1, d, h, w]);
    for z in 0..d {
        for y in 0..h {
            for x in 0..w {
                let src = ((z * h + y) * w + x) * volume.samples;
                out[[0, 0, z, y, x]] = flat[src];
            }
        }
    }
    out
}

/// Extract a single 2D axial slice as a 4D NCHW tensor (depth = 1).
pub fn slice_to_nchw(volume: &Volume, z: usize, norm: Normalization) -> ArrayD<f32> {
    let [w, h, _d] = volume.dims;
    let mut flat = vec![0.0f32; w * h];
    for y in 0..h {
        for x in 0..w {
            let src = ((z * h + y) * w + x) * volume.samples;
            flat[y * w + x] = volume.data[src];
        }
    }
    apply_normalization(&mut flat, norm);
    let mut out = ArrayD::<f32>::zeros(vec![1, 1, h, w]);
    for y in 0..h {
        for x in 0..w {
            out[[0, 0, y, x]] = flat[y * w + x];
        }
    }
    out
}

fn apply_normalization(data: &mut [f32], norm: Normalization) {
    match norm {
        Normalization::None => {}
        Normalization::MinMax => {
            let min = data.iter().cloned().fold(f32::INFINITY, f32::min);
            let max = data.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
            let range = (max - min).max(1e-8);
            for v in data.iter_mut() {
                *v = (*v - min) / range;
            }
        }
        Normalization::ZScore => {
            let n = data.len() as f32;
            let mean = data.iter().sum::<f32>() / n;
            let var = data.iter().map(|v| (v - mean).powi(2)).sum::<f32>() / n;
            let std = var.sqrt().max(1e-8);
            for v in data.iter_mut() {
                *v = (*v - mean) / std;
            }
        }
    }
}
