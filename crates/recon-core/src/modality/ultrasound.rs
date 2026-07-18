//! Ultrasound reconstruction (stub).
//!
//! Placeholder module for beam-formed ultrasound image reconstruction. The full
//! implementation (delay-and-sum beamforming, envelope detection) is planned for
//! a later phase. This stub documents the intended interface.

use crate::geometry::Projection;
use crate::modality::ReconError;
use crate::modality::Result;
use crate::volume::Volume;

/// Ultrasound reconstruction parameters (placeholder).
#[derive(Debug, Clone, Default)]
pub struct UltrasoundParams {
    pub num_elements: usize,
    pub pitch_mm: f32,
    pub speed_of_sound: f32,
}

/// Reconstruct an ultrasound B-mode volume from channel data.
///
/// **Stub:** returns an error until the beamformer is implemented.
pub fn reconstruct(_channels: &[Projection], _params: &UltrasoundParams) -> Result<Volume> {
    Err(ReconError::UnsupportedModality(
        "Ultrasound reconstruction is not yet implemented".to_string(),
    ))
}
