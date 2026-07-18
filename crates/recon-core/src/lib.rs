//! # spectra-recon-core
//!
//! Shared reconstruction core: data model, volumes, modality trait, dispatch.

pub mod geometry;
pub mod modality;
pub mod volume;

pub use geometry::{CtGeometry, MriGeometry, Projection};
pub use modality::{
    reconstruct_ct, reconstruct_mri, CtFilter, CtParams, Modality, ReconError, Reconstructor,
    Result,
};
pub use volume::Volume;
