//! # spectra-ai-bridge
//!
//! AI diagnostic bridge: ONNX runtime integration, model registry, adapters.

pub mod adapter;
pub mod inference;
pub mod output;
pub mod registry;

pub use adapter::{slice_to_nchw, to_nchw, Normalization};
pub use inference::{AiError, Diagnostic, DiagnosticModel};
pub use output::{annotations, parse_scores};
pub use registry::{ModelDescriptor, ModelRegistry, ModelTask, NormalizationKind};
