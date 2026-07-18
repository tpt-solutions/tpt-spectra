//! Micro-benchmark for CT filtered back-projection.
//!
//! Run with: `cargo run -p spectra-recon-core --example bench --release`

use spectra_recon_core::geometry::{CtGeometry, Projection};
use spectra_recon_core::modality::{reconstruct_ct, CtFilter, CtParams};
use std::time::Instant;

fn main() {
    let det_cols = 256;
    let views = 360;
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

    let mut projections = Vec::with_capacity(views);
    let a = det_cols as f32 / 4.0;
    for v in 0..views {
        let angle = std::f32::consts::PI * v as f32 / views as f32;
        let ca = angle.cos();
        let sa = angle.sin();
        let mut data = vec![0.0f32; det_cols];
        for (col, s) in data.iter_mut().enumerate() {
            let t = col as f32 - (det_cols as f32 - 1.0) / 2.0;
            let inside = (t * ca / a).powi(2) + (t * sa / a).powi(2);
            *s = if inside <= 1.0 {
                (1.0 - inside).sqrt()
            } else {
                0.0
            };
        }
        projections.push(Projection::new(1, det_cols, data));
    }

    let iters = 5;
    let start = Instant::now();
    for _ in 0..iters {
        let _ = reconstruct_ct(&projections, &params).unwrap();
    }
    let elapsed = start.elapsed();
    println!(
        "CT FBP: {}x{} detector, {} views -> {:.2?} per recon (avg over {iters})",
        det_cols,
        det_cols,
        views,
        elapsed / iters
    );
}
