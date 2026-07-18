//! AI bridge tests that do not require a real ONNX model file.

use ndarray::ArrayD;
use spectra_ai_bridge::adapter::{slice_to_nchw, to_nchw, Normalization};
use spectra_ai_bridge::output::{annotations, parse_scores};
use spectra_ai_bridge::registry::{ModelRegistry, ModelTask, NormalizationKind};
use spectra_recon_core::Volume;

fn make_volume() -> Volume {
    // 4x4x2 volume with a gradient.
    let mut v = Volume::new([4, 4, 2], [1.0, 1.0, 1.0]);
    for z in 0..2 {
        for y in 0..4 {
            for x in 0..4 {
                v.set(x, y, z, 0, (x + y + z) as f32);
            }
        }
    }
    v
}

#[test]
fn to_nchw_shape_is_5d_ncdhw() {
    let v = make_volume();
    let t: ArrayD<f32> = to_nchw(&v, Normalization::None);
    assert_eq!(t.shape(), &[1, 1, 2, 4, 4]);
}

#[test]
fn slice_to_nchw_shape_is_4d_nchw() {
    let v = make_volume();
    let t = slice_to_nchw(&v, 0, Normalization::None);
    assert_eq!(t.shape(), &[1, 1, 4, 4]);
}

#[test]
fn minmax_normalization_scales_to_unit_range() {
    let v = make_volume();
    let t: ArrayD<f32> = to_nchw(&v, Normalization::MinMax);
    let min = t.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = t.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    assert!((min - 0.0).abs() < 1e-6, "min should be 0, got {min}");
    assert!((max - 1.0).abs() < 1e-6, "max should be 1, got {max}");
}

#[test]
fn registry_examples_present() {
    let reg = ModelRegistry::with_examples();
    assert!(reg.get("tumor-detection-v1").is_some());
    assert!(reg.get("fracture-mapping-v1").is_some());
    let tumor = reg.get("tumor-detection-v1").unwrap();
    assert_eq!(tumor.task, ModelTask::TumorDetection);
    assert_eq!(tumor.normalization, NormalizationKind::MinMax);
}

#[test]
fn parse_scores_and_annotations() {
    let diag = parse_scores(&[0.1, 0.85, 0.05], &["healthy", "tumor", "artifact"]);
    assert!((diag.confidence - 0.85).abs() < 1e-6);
    assert_eq!(diag.classes[1], "tumor");
    let ann = annotations(&diag, 0.5);
    assert!(ann.iter().any(|a| a.contains("tumor")));
}
