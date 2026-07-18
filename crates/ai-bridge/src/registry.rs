//! Diagnostic model registry: metadata describing third-party ONNX models.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The clinical task a diagnostic model performs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelTask {
    /// Detect and localize tumors.
    TumorDetection,
    /// Map fractures in bone.
    FractureMapping,
    /// Arbitrary / user-defined task.
    Custom,
}

/// Descriptor for a registered ONNX model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDescriptor {
    pub id: String,
    pub name: String,
    pub task: ModelTask,
    /// Filesystem path to the `.onnx` model.
    pub path: String,
    /// Expected input tensor shape (NCHW or NCHW-3D).
    pub input_shape: Vec<usize>,
    /// Normalization to apply to input volumes.
    pub normalization: NormalizationKind,
    /// Whether the model expects a 2D slice (true) or full 3D volume (false).
    pub expects_2d: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NormalizationKind {
    MinMax,
    ZScore,
    None,
}

/// A registry of diagnostic models loaded from configuration or code.
#[derive(Debug, Default)]
pub struct ModelRegistry {
    models: HashMap<String, ModelDescriptor>,
}

impl ModelRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a model descriptor.
    pub fn register(&mut self, desc: ModelDescriptor) {
        self.models.insert(desc.id.clone(), desc);
    }

    /// Look up a model by id.
    pub fn get(&self, id: &str) -> Option<&ModelDescriptor> {
        self.models.get(id)
    }

    /// All registered model ids.
    pub fn ids(&self) -> Vec<&str> {
        self.models.keys().map(|s| s.as_str()).collect()
    }

    /// Example: register the bundled tumor-detection model descriptor.
    pub fn with_examples() -> Self {
        let mut reg = Self::new();
        reg.register(ModelDescriptor {
            id: "tumor-detection-v1".into(),
            name: "Tumor Detection (example)".into(),
            task: ModelTask::TumorDetection,
            path: "models/tumor_detection.onnx".into(),
            input_shape: vec![1, 1, 64, 64],
            normalization: NormalizationKind::MinMax,
            expects_2d: true,
        });
        reg.register(ModelDescriptor {
            id: "fracture-mapping-v1".into(),
            name: "Fracture Mapping (example)".into(),
            task: ModelTask::FractureMapping,
            path: "models/fracture_mapping.onnx".into(),
            input_shape: vec![1, 1, 64, 64],
            normalization: NormalizationKind::ZScore,
            expects_2d: true,
        });
        reg
    }
}
