//! Phantom-based verification: reconstruct synthetic phantoms and assert
//! known-good structural properties. These tests form the foundation of the
//! Phase 6 verification suite (pixel-perfect parity against DCMTK is added
//! separately, gated on the `dcmtk` tool being available).

use spectra_recon_core::geometry::{CtGeometry, Projection};
use spectra_recon_core::modality::{reconstruct_ct, CtFilter, CtParams};

/// A Shepp-Logan-like elliptical phantom projected into a sinogram.
fn ellipse_sinogram(views: usize, det_cols: usize) -> Vec<Projection> {
    let mut out = Vec::with_capacity(views);
    let a = det_cols as f32 / 4.0; // semi-axis
    for v in 0..views {
        let angle = std::f32::consts::PI * v as f32 / views as f32;
        let ca = angle.cos();
        let sa = angle.sin();
        let mut data = vec![0.0f32; det_cols];
        for (col, s) in data.iter_mut().enumerate() {
            let t = col as f32 - (det_cols as f32 - 1.0) / 2.0;
            // Line integral of an ellipse along direction (ca,sa) at offset t.
            let inside = (t * ca / a).powi(2) + (t * sa / a).powi(2);
            *s = if inside <= 1.0 {
                (1.0 - inside).sqrt()
            } else {
                0.0
            };
        }
        out.push(Projection::new(1, det_cols, data));
    }
    out
}

#[test]
fn phantom_reconstruction_is_finite_and_centered() {
    let det_cols = 96;
    let views = 120;
    let geo = CtGeometry {
        num_views: views,
        detector: [1, det_cols],
        sid: 500.0,
        sdd: 1000.0,
        pixel_size: [1.0, 1.0],
        fov: det_cols as f32,
    };
    let params = CtParams {
        geometry: geo,
        filter: CtFilter::RamLak,
    };
    let sino = ellipse_sinogram(views, det_cols);
    let vol = reconstruct_ct(&sino, &params).expect("reconstruct");
    assert_eq!(vol.dims, [det_cols, det_cols, 1]);
    assert!(vol.data.iter().all(|v| v.is_finite()));

    let c = det_cols / 2;
    let center = vol.get(c, c, 0, 0);
    assert!(
        center.is_finite() && center.abs() < 1e4,
        "center sane: {center}"
    );

    // A centered phantom must reconstruct to an approximately centrally
    // symmetric volume (small asymmetries from edge clamping in BP).
    for y in 0..det_cols {
        for x in 0..det_cols {
            let v = vol.get(x, y, 0, 0);
            let sym = vol.get(det_cols - 1 - x, det_cols - 1 - y, 0, 0);
            assert!(
                (v - sym).abs() < 0.05,
                "asymmetry at ({x},{y}): {v} vs {sym}"
            );
        }
    }
}
