//! Output handling: turn raw model tensors into diagnostic confidence scores.

use crate::inference::Diagnostic;

/// Convert a raw 1D/probability output tensor into a [`Diagnostic`].
///
/// `scores` is interpreted as a vector of per-class probabilities. When
/// `classes` is shorter than `scores`, generic `class_N` labels are used.
pub fn parse_scores(scores: &[f32], classes: &[&str]) -> Diagnostic {
    let classes: Vec<String> = (0..scores.len())
        .map(|i| {
            classes
                .get(i)
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("class_{i}"))
        })
        .collect();
    let confidence = scores.iter().cloned().fold(0.0f32, f32::max);
    Diagnostic {
        classes,
        scores: scores.to_vec(),
        confidence,
    }
}

/// Build human-readable annotation strings from a diagnostic (for overlays).
pub fn annotations(diag: &Diagnostic, threshold: f32) -> Vec<String> {
    diag.scores
        .iter()
        .enumerate()
        .filter(|(_, &s)| s >= threshold)
        .map(|(i, &s)| format!("{}: {:.1}%", diag.classes[i], s * 100.0))
        .collect()
}
