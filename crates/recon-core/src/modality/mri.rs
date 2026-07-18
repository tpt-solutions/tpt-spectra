//! MRI iterative reconstruction from k-space.
//!
//! A working reference implementation: zero-filled k-space is inverse Fourier
//! transformed (per-slice 2D FFT) and, when undersampled, a few iterations of
//! a simple soft-threshold (ISTA-style) denoising pass are applied. This is a
//! lightweight stand-in for a full compressed-sensing pipeline.

use crate::geometry::MriGeometry;
use crate::modality::{ReconError, Result};
use crate::volume::Volume;

/// Iterative MRI reconstructor (FFT + soft-threshold refinement).
#[derive(Default)]
pub struct IterativeReconstruction;

impl IterativeReconstruction {
    pub fn new() -> Self {
        Self
    }
}

/// Reconstruct an MRI volume from a set of k-space slices (one Vec<f32> per slice,
/// each of length `nx * ny`).
pub fn iterative_reconstruction(kspace: &[Vec<f32>], geometry: &MriGeometry) -> Result<Volume> {
    let [nx, ny, nz] = geometry.matrix;
    if kspace.len() != nz {
        return Err(ReconError::InvalidGeometry(format!(
            "expected {} k-space slices, got {}",
            nz,
            kspace.len()
        )));
    }
    for (i, k) in kspace.iter().enumerate() {
        if k.len() != nx * ny {
            return Err(ReconError::InvalidGeometry(format!(
                "k-space slice {i} length {} != nx*ny {}",
                k.len(),
                nx * ny
            )));
        }
    }

    let spacing = [1.0, 1.0, 1.0];
    let mut volume = Volume::new([nx, ny, nz], spacing);

    for (z, kslice) in kspace.iter().enumerate().take(nz) {
        let slice = ifft2(kslice, nx, ny);
        let refined = ista_refine(&slice, nx, ny, geometry.acceleration);
        for y in 0..ny {
            for x in 0..nx {
                let i = (z * ny + y) * nx + x;
                volume.data[i] = refined[y * nx + x];
            }
        }
    }

    Ok(volume)
}

/// Inverse 2D FFT of a real-valued "k-space" (treated as the magnitude spectrum
/// with Hermitian symmetry assumed) using a naive DFT. Adequate for small
/// phantom sizes used in tests.
fn ifft2(kspace: &[f32], nx: usize, ny: usize) -> Vec<f32> {
    let mut out = vec![0.0f32; nx * ny];
    let n = (nx * ny) as f32;
    for y in 0..ny {
        for x in 0..nx {
            let mut re = 0.0f32;
            let mut im = 0.0f32;
            for ky in 0..ny {
                for kx in 0..nx {
                    let phase = -2.0
                        * std::f32::consts::PI
                        * ((kx as f32 * x as f32 / nx as f32) + (ky as f32 * y as f32 / ny as f32));
                    let amp = kspace[ky * nx + kx];
                    re += amp * phase.cos();
                    im += amp * phase.sin();
                }
            }
            out[y * nx + x] = (re / n).hypot(im / n);
        }
    }
    out
}

/// A few iterations of ISTA-style soft-thresholding to mimic iterative
/// reconstruction regularization. `acceleration > 1` increases the threshold.
fn ista_refine(slice: &[f32], nx: usize, ny: usize, acceleration: f32) -> Vec<f32> {
    let mut x = slice.to_vec();
    let lambda = 0.02 * acceleration.max(1.0);
    for _ in 0..4 {
        let mut next = vec![0.0f32; nx * ny];
        for i in 0..x.len() {
            let v = x[i];
            // gradient-step toward data fidelity (here identity) + soft threshold.
            let grad = v - slice[i];
            let updated = v - 0.5 * grad;
            next[i] = soft_threshold(updated, lambda);
        }
        x = next;
    }
    x
}

fn soft_threshold(v: f32, lambda: f32) -> f32 {
    if v > lambda {
        v - lambda
    } else if v < -lambda {
        v + lambda
    } else {
        0.0
    }
}
