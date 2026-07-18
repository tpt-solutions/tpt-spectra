//! PET reconstruction (stub).
//!
//! Placeholder module for PET list-mode / sinogram OSEM reconstruction. The full
//! implementation (forward/back projection, OSEM iterations) is planned for a
//! later phase. This stub documents the intended interface.

use crate::geometry::Projection;
use crate::modality::ReconError;
use crate::modality::Result;
use crate::volume::Volume;

/// PET reconstruction parameters (placeholder).
#[derive(Debug, Clone, Default)]
pub struct PetParams {
    pub num_detector_rings: usize,
    pub voxel_size_mm: f32,
    pub iterations: usize,
    pub subsets: usize,
}

/// Reconstruct a PET activity volume from sinogram projections.
///
/// **Stub:** returns an error until OSEM is implemented.
pub fn reconstruct(_sinograms: &[Projection], _params: &PetParams) -> Result<Volume> {
    Err(ReconError::UnsupportedModality(
        "PET reconstruction is not yet implemented".to_string(),
    ))
}
