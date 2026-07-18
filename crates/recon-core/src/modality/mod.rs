//! Common reconstruction trait and engine dispatch used to run modalities.

pub mod ct;
pub mod mri;
pub mod pet;
pub mod ultrasound;

use crate::geometry::{CtGeometry, MriGeometry, Projection};
use crate::volume::Volume;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReconError {
    #[error("unsupported modality: {0}")]
    UnsupportedModality(String),
    #[error("invalid geometry: {0}")]
    InvalidGeometry(String),
    #[error("projection data length mismatch for view {view}: expected {expected}, got {got}")]
    ProjectionMismatch {
        view: usize,
        expected: usize,
        got: usize,
    },
    #[error("numeric error: {0}")]
    Numeric(String),
}

pub type Result<T> = std::result::Result<T, ReconError>;

/// Identifies a supported imaging modality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modality {
    CT,
    MRI,
    Ultrasound,
    PET,
}

impl Modality {
    pub fn name(&self) -> &'static str {
        match self {
            Modality::CT => "CT",
            Modality::MRI => "MRI",
            Modality::Ultrasound => "Ultrasound",
            Modality::PET => "PET",
        }
    }
}

/// A reconstructor takes raw projections / k-space and produces a volume.
pub trait Reconstructor {
    /// Human-readable name of the algorithm.
    fn name(&self) -> &'static str;

    /// Reconstruct from a stack of projections (row-major per view).
    fn reconstruct_projections(
        &self,
        views: &[Projection],
        output_dims: [usize; 3],
    ) -> Result<Volume>;
}

/// CT reconstruction parameters.
#[derive(Debug, Clone)]
pub struct CtParams {
    pub geometry: CtGeometry,
    /// Use aRam-Lak (default) or Shepp-Logan filter for FBP.
    pub filter: CtFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CtFilter {
    RamLak,
    SheppLogan,
}

/// Dispatch entry point: reconstruct a CT volume from projections.
pub fn reconstruct_ct(views: &[Projection], params: &CtParams) -> Result<Volume> {
    let output_dims = [
        params.geometry.detector[1],
        params.geometry.detector[1],
        params.geometry.detector[0].max(1),
    ];
    crate::modality::ct::filtered_back_projection(views, params, output_dims)
}

/// Dispatch entry point: reconstruct an MRI volume from k-space.
pub fn reconstruct_mri(kspace: &[Vec<f32>], geometry: &MriGeometry) -> Result<Volume> {
    crate::modality::mri::iterative_reconstruction(kspace, geometry)
}
