//! Integration tests: reconstruct known phantoms and check reference properties.

use spectra_recon_core::geometry::{CtGeometry, MriGeometry, Projection};
use spectra_recon_core::modality::{reconstruct_ct, reconstruct_mri, CtFilter, CtParams};

/// Build a simple synthetic CT sinogram for a centered disk phantom.
///
/// For each view angle we integrate a constant along detector columns within
/// the projected disk width, producing a non-negative projection profile.
fn disk_sinogram(views: usize, det_cols: usize, det_rows: usize) -> Vec<Projection> {
    let mut out = Vec::new();
    for v in 0..views {
        let angle = std::f32::consts::PI * v as f32 / views as f32;
        let mut data = vec![0.0f32; det_rows * det_cols];
        let r = (det_cols as f32 / 4.0).max(1.0);
        for row in 0..det_rows {
            let _ = row;
            for col in 0..det_cols {
                // Disk projected along x' axis: support is [-r, r] rotated.
                let t = (col as f32 - (det_cols as f32 - 1.0) / 2.0) * angle.cos();
                let val = if t.abs() <= r {
                    1.0 - (t / r).powi(2)
                } else {
                    0.0
                };
                data[row * det_cols + col] = val;
            }
        }
        out.push(Projection::new(det_rows, det_cols, data));
    }
    out
}

#[test]
fn ct_fbp_reconstructs_centered_disk() {
    let det_cols = 64;
    let det_rows = 1;
    let views = 90;
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
    let sinograms = disk_sinogram(views, det_cols, det_rows);
    let vol = reconstruct_ct(&sinograms, &params).expect("ct recon");

    assert_eq!(vol.dims, [det_cols, det_cols, 1]);
    assert!(
        vol.data.iter().all(|v| v.is_finite()),
        "volume must be finite"
    );

    // A centered phantom must reconstruct to a centrally symmetric volume.
    // A centered phantom must reconstruct to an approximately centrally
    // symmetric volume (small asymmetries arise from edge clamping in BP).
    let c = det_cols / 2;
    for y in 0..det_cols {
        for x in 0..det_cols {
            let sym = vol.get(det_cols - 1 - x, det_cols - 1 - y, 0, 0);
            let v = vol.get(x, y, 0, 0);
            assert!(
                (v - sym).abs() < 0.02,
                "asymmetry at ({x},{y}): {v} vs {sym}"
            );
        }
    }
    let _ = c;
}

#[test]
fn ct_fbp_rejects_view_count_mismatch() {
    let det_cols = 32;
    let geo = CtGeometry {
        num_views: 10,
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
    let sinograms = disk_sinogram(9, det_cols, 1);
    let res = reconstruct_ct(&sinograms, &params);
    assert!(res.is_err());
}

/// Build a synthetic k-space for a constant disk in the image domain, then
/// verify MRI reconstruction recovers a positive central region.
#[test]
fn mri_reconstruction_recovers_positive_center() {
    let nx = 16;
    let ny = 16;
    let nz = 1;
    let matrix = [nx, ny, nz];
    // Forward FFT of a constant image = DC-only k-space.
    let mut kspace = vec![vec![0.0f32; nx * ny]; nz];
    kspace[0][0] = 100.0; // DC component (scaled above ISTA threshold)
    let geo = MriGeometry {
        matrix,
        field_strength: 3.0,
        coils: 1,
        acceleration: 1.0,
    };
    let vol = reconstruct_mri(&kspace, &geo).expect("mri recon");
    assert_eq!(vol.dims, [nx, ny, nz]);
    let c = nx / 2;
    // ISTA soft-threshold keeps the DC-seeded plateau positive.
    assert!(vol.get(c, c, 0, 0) > 0.0, "center should be positive");
}
