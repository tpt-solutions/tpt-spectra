//! Projection geometry descriptors shared by reconstruction algorithms.

use serde::{Deserialize, Serialize};

/// A single projection view in a scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Projection {
    /// Detector row count (along the in-plane vertical axis).
    pub rows: usize,
    /// Detector column count (along the in-plane horizontal axis).
    pub cols: usize,
    /// Raw detector samples (row-major, length = rows * cols).
    pub data: Vec<f32>,
}

impl Projection {
    pub fn new(rows: usize, cols: usize, data: Vec<f32>) -> Self {
        assert_eq!(data.len(), rows * cols, "projection data length mismatch");
        Self { rows, cols, data }
    }

    pub fn at(&self, row: usize, col: usize) -> f32 {
        self.data[row * self.cols + col]
    }
}

/// Cone-beam / fan-beam CT scanner geometry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CtGeometry {
    /// Number of projection angles.
    pub num_views: usize,
    /// Detector dimensions [rows, cols].
    pub detector: [usize; 2],
    /// Source-to-isocenter distance (mm).
    pub sid: f32,
    /// Source-to-detector distance (mm).
    pub sdd: f32,
    /// Pixel size on the detector [row, col] (mm).
    pub pixel_size: [f32; 2],
    /// Reconstruction field-of-view diameter (mm).
    pub fov: f32,
}

/// MRI acquisition description (k-space sampling).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MriGeometry {
    /// Matrix size [nx, ny, nz].
    pub matrix: [usize; 3],
    /// Field strength (Tesla).
    pub field_strength: f32,
    /// Number of coils (parallel imaging).
    pub coils: usize,
    /// k-space undersampling factor (1.0 = fully sampled).
    pub acceleration: f32,
}
