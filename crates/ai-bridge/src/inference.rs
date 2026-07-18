//! ONNX Runtime inference wrapper for diagnostic models.

use crate::adapter::{slice_to_nchw, to_nchw, Normalization, NormalizationKindExt};
use crate::registry::ModelDescriptor;
use ndarray::ArrayD;
use ort::session::Session;
use spectra_recon_core::Volume;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("model file not found: {0}")]
    ModelNotFound(String),
    #[error("onnx runtime error: {0}")]
    Ort(String),
    #[error("output tensor missing or malformed: {0}")]
    BadOutput(String),
    #[error("input shape mismatch: expected {expected:?}, got {got:?}")]
    ShapeMismatch {
        expected: Vec<usize>,
        got: Vec<usize>,
    },
}

pub type Result<T> = std::result::Result<T, AiError>;

/// A loaded diagnostic model ready to run inference.
pub struct DiagnosticModel {
    session: Session,
    descriptor: ModelDescriptor,
}

impl DiagnosticModel {
    /// Load a model from its descriptor.
    pub fn load(desc: &ModelDescriptor) -> Result<Self> {
        let session = Session::builder()
            .map_err(|e| AiError::Ort(e.to_string()))?
            .commit_from_file(&desc.path)
            .map_err(|e| AiError::Ort(e.to_string()))?;
        Ok(Self {
            session,
            descriptor: desc.clone(),
        })
    }

    pub fn descriptor(&self) -> &ModelDescriptor {
        &self.descriptor
    }

    fn norm(&self) -> Normalization {
        self.descriptor.normalization.to_norm()
    }

    /// Run inference on a reconstructed volume. Returns the raw output tensor.
    pub fn infer(&mut self, volume: &Volume) -> Result<ArrayD<f32>> {
        let tensor = if self.descriptor.expects_2d {
            // Use the central axial slice.
            let z = volume.dims[2] / 2;
            slice_to_nchw(volume, z, self.norm())
        } else {
            to_nchw(volume, self.norm())
        };

        let shape = tensor.shape().to_vec();
        let expected = &self.descriptor.input_shape;
        if !shapes_compatible(expected, &shape) {
            return Err(AiError::ShapeMismatch {
                expected: expected.clone(),
                got: shape,
            });
        }

        let input =
            ort::value::Tensor::from_array(tensor).map_err(|e| AiError::Ort(e.to_string()))?;
        let input_name = self
            .session
            .inputs()
            .first()
            .map(|o| o.name().to_string())
            .ok_or_else(|| AiError::BadOutput("no model inputs".into()))?;

        let outputs = self
            .session
            .run(ort::inputs![input_name.as_str() => input])
            .map_err(|e| AiError::Ort(e.to_string()))?;

        let (_name, value) = outputs
            .iter()
            .next()
            .ok_or_else(|| AiError::BadOutput("empty outputs".into()))?;

        let arr = value
            .try_extract_array::<f32>()
            .map_err(|e| AiError::BadOutput(e.to_string()))?;
        // `arr` borrows from `value`; clone into an owned ArrayD.
        let owned: ArrayD<f32> = arr.to_owned();
        Ok(owned)
    }
}

fn shapes_compatible(expected: &[usize], got: &[usize]) -> bool {
    if expected.len() != got.len() {
        return false;
    }
    expected.iter().zip(got).all(|(e, g)| *e == *g || *e == 0)
}

/// A diagnostic result with per-class confidence scores.
#[derive(Debug, Clone)]
pub struct Diagnostic {
    /// Class names (or empty if unknown).
    pub classes: Vec<String>,
    /// Confidence score per class, in [0, 1].
    pub scores: Vec<f32>,
    /// Max confidence across classes.
    pub confidence: f32,
}
