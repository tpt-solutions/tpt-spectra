//! # spectra-gpu-backend
//!
//! GPU reconstruction backend (wgpu): device init, pipelines, compute shaders.

pub mod device;
pub mod shader;

pub use device::{GpuContext, GpuError, Result};
pub use shader::{ComputeKernel, BACKPROJECT_WGSL};
