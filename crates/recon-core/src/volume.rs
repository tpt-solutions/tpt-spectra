//! Shared volume / voxel data structures used across all reconstruction modalities.

use serde::{Deserialize, Serialize};

/// A 3D scalar or vector volume stored in a flat, row-major buffer.
///
/// Data is laid out as `[(z * rows + y) * cols + x]` for `samples == 1`, with
/// an outer `samples` stride for multi-component (e.g. RGB) volumes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub dims: [usize; 3],
    pub samples: usize,
    pub spacing: [f32; 3],
    pub data: Vec<f32>,
}

impl Volume {
    /// Allocate a zero-initialized scalar volume.
    pub fn new(dims: [usize; 3], spacing: [f32; 3]) -> Self {
        let len = dims[0] * dims[1] * dims[2];
        Self {
            dims,
            samples: 1,
            spacing,
            data: vec![0.0; len],
        }
    }

    /// Allocate a zero-initialized multi-sample volume.
    pub fn with_samples(dims: [usize; 3], samples: usize, spacing: [f32; 3]) -> Self {
        let len = dims[0] * dims[1] * dims[2] * samples;
        Self {
            dims,
            samples,
            spacing,
            data: vec![0.0; len],
        }
    }

    /// Linear index into `data` for a voxel coordinate (no bounds checking).
    pub fn index(&self, x: usize, y: usize, z: usize, s: usize) -> usize {
        ((z * self.dims[1] + y) * self.dims[0] + x) * self.samples + s
    }

    /// Read a sample value.
    pub fn get(&self, x: usize, y: usize, z: usize, s: usize) -> f32 {
        self.data[self.index(x, y, z, s)]
    }

    /// Write a sample value.
    pub fn set(&mut self, x: usize, y: usize, z: usize, s: usize, v: f32) {
        let i = self.index(x, y, z, s);
        self.data[i] = v;
    }

    /// Total number of voxels (ignoring `samples`).
    pub fn total_voxels(&self) -> usize {
        self.dims[0] * self.dims[1] * self.dims[2]
    }

    /// Number of `f32` values in the backing buffer.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}
