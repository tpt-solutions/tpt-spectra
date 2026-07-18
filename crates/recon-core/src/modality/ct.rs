//! CT filtered back-projection (FBP).
//!
//! A working, reference-quality CPU implementation suitable for small phantom
//! volumes and integration tests. Uses a ramp (Ram-Lak) or Shepp-Logan filter
//! in the projection domain followed by weighted back-projection.

use crate::modality::{CtFilter, CtParams, ReconError, Result};
use crate::volume::Volume;
use crate::{Projection, Reconstructor};

/// Filtered back-projection reconstructor.
pub struct FilteredBackProjection {
    params: CtParams,
}

impl FilteredBackProjection {
    pub fn new(params: CtParams) -> Self {
        Self { params }
    }
}

impl Reconstructor for FilteredBackProjection {
    fn name(&self) -> &'static str {
        match self.params.filter {
            CtFilter::RamLak => "CT Filtered Back-Projection (Ram-Lak)",
            CtFilter::SheppLogan => "CT Filtered Back-Projection (Shepp-Logan)",
        }
    }

    fn reconstruct_projections(
        &self,
        views: &[Projection],
        output_dims: [usize; 3],
    ) -> Result<Volume> {
        filtered_back_projection(views, &self.params, output_dims)
    }
}

/// Apply a 1D ramp/Shepp-Logan filter to each detector row of a projection,
/// Build the frequency-domain ramp filter kernel (length n, symmetric).
#[allow(clippy::needless_range_loop)]
fn filter_projection(row: &[f32], filter: CtFilter) -> Vec<f32> {
    let n = row.len();
    // Build the frequency-domain ramp filter kernel (length n, symmetric).
    let mut kernel = vec![0.0f32; n];
    let half = n / 2;
    for i in 0..n {
        let k = i as isize - half as isize;
        if k == 0 {
            kernel[i] = 0.25; // |k| at DC-ish center (normalized)
        } else if k % 2 == 1 {
            kernel[i] = -1.0 / ((std::f32::consts::PI * k as f32).powi(2));
        } else {
            kernel[i] = 0.0;
        }
    }
    if filter == CtFilter::SheppLogan {
        // Apodize with a sinc window to reduce high-frequency noise.
        for i in 0..n {
            let k = i as isize - half as isize;
            let w = (std::f32::consts::PI * k as f32 / half as f32).sin()
                / (std::f32::consts::PI * k as f32 / half as f32).max(1e-6);
            kernel[i] *= w;
        }
    }
    convolve_circular(row, &kernel)
}

/// Circular convolution of `signal` with `kernel` (both length n).
#[allow(clippy::needless_range_loop)]
fn convolve_circular(signal: &[f32], kernel: &[f32]) -> Vec<f32> {
    let n = signal.len();
    let mut out = vec![0.0f32; n];
    for i in 0..n {
        let mut acc = 0.0f32;
        for j in 0..n {
            let idx = (i as isize - j as isize).rem_euclid(n as isize) as usize;
            acc += signal[j] * kernel[idx];
        }
        out[i] = acc;
    }
    out
}

/// Core FBP routine.
pub fn filtered_back_projection(
    views: &[Projection],
    params: &CtParams,
    output_dims: [usize; 3],
) -> Result<Volume> {
    let geo = &params.geometry;
    let [od_rows, od_cols, od_slices] = output_dims;
    if views.len() != geo.num_views {
        return Err(ReconError::InvalidGeometry(format!(
            "expected {} views, got {}",
            geo.num_views,
            views.len()
        )));
    }
    let (det_rows, det_cols) = (geo.detector[0], geo.detector[1]);
    let spacing = [geo.pixel_size[1], geo.pixel_size[1], geo.pixel_size[0]];

    let mut volume = Volume::new([od_rows, od_cols, od_slices], spacing);
    let filter = params.filter;

    for (v, proj) in views.iter().enumerate() {
        if proj.data.len() != det_rows * det_cols {
            return Err(ReconError::ProjectionMismatch {
                view: v,
                expected: det_rows * det_cols,
                got: proj.data.len(),
            });
        }
        // Filter each detector row (treating rows as sinogram lines).
        let mut filtered = vec![0.0f32; det_rows * det_cols];
        for r in 0..det_rows {
            let row = &proj.data[r * det_cols..(r + 1) * det_cols];
            let f = filter_projection(row, filter);
            filtered[r * det_cols..(r + 1) * det_cols].copy_from_slice(&f);
        }

        // Back-project: simple parallel-beam back-projection over a rotated grid.
        let angle = if geo.num_views > 1 {
            std::f32::consts::PI * v as f32 / geo.num_views as f32
        } else {
            0.0
        };
        let cos = angle.cos();
        let sin = angle.sin();
        let cx = (od_cols as f32 - 1.0) / 2.0;
        let cy = (od_rows as f32 - 1.0) / 2.0;

        for y in 0..od_rows {
            for x in 0..od_cols {
                // Map image coordinate into detector column space.
                let dx = x as f32 - cx;
                let dy = y as f32 - cy;
                let t = dx * cos + dy * sin; // projection coordinate
                let col_f = t / geo.pixel_size[1] + cx;
                let col0 = col_f.floor() as i32;
                let col1 = col0 + 1;
                let frac = col_f - col0 as f32;
                // Use the central detector row for this 2D slice back-projection.
                let rr = (det_rows / 2).min(det_rows.saturating_sub(1));
                let sample = |c: i32| -> f32 {
                    if c < 0 || c >= det_cols as i32 {
                        0.0
                    } else {
                        filtered[rr * det_cols + c as usize]
                    }
                };
                let val = sample(col0) * (1.0 - frac) + sample(col1) * frac;
                // Write into slice 0 (single-slice phantom support).
                let z = (v * od_slices / geo.num_views.max(1)).min(od_slices - 1);
                let idx = (z * od_rows + y) * od_cols + x;
                volume.data[idx] += val;
            }
        }
    }

    // Normalize by number of views.
    let inv = 1.0 / geo.num_views.max(1) as f32;
    for v in volume.data.iter_mut() {
        *v *= inv;
    }

    Ok(volume)
}
