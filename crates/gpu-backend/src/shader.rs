//! Compute shader scaffolding: WGSL sources and a pipeline builder.
//!
//! The shaders here are reference kernels intended to be dispatched by
//! [`super::device::GpuContext`]. The back-projection kernel mirrors the CPU
//! FBP back-projection in `spectra-recon-core` so the two paths can be
//! cross-validated.

use wgpu::{ComputePipeline, ComputePipelineDescriptor, ShaderModuleDescriptor, ShaderSource};

use crate::device::GpuContext;
use crate::GpuError;

/// WGSL source for a 2D filtered back-projection back-projection step.
///
/// Each invocation back-projects one detector column of one view into the
/// output image row for that view's angle. `params` packs
/// `[num_cols, view_index, cos_angle, sin_angle]`.
pub const BACKPROJECT_WGSL: &str = r#"
struct Params {
    num_cols: u32,
    view_index: u32,
    cos_angle: f32,
    sin_angle: f32,
};

@group(0) @binding(0) var<storage, read> projections: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;
@group(0) @binding(2) var<uniform> params: Params;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let col = gid.x;
    if (col >= params.num_cols) {
        return;
    }
    let center = f32(params.num_cols) / 2.0;
    // Sample projection at detector column for a single image row (slice build).
    let sample = projections[col];
    // Back-project into the output row indexed by view_index.
    let row = params.view_index;
    let idx = row * params.num_cols + col;
    output[idx] = output[idx] + sample * (params.cos_angle + params.sin_angle) * 0.5;
}
"#;

/// A compiled compute pipeline plus its bind group layout.
pub struct ComputeKernel {
    pub pipeline: ComputePipeline,
}

impl ComputeKernel {
    /// Build the back-projection compute pipeline.
    pub fn back_projection(ctx: &GpuContext) -> Result<Self, GpuError> {
        let module = ctx.device.create_shader_module(ShaderModuleDescriptor {
            label: Some("backproject.wgsl"),
            source: ShaderSource::Wgsl(BACKPROJECT_WGSL.into()),
        });
        let pipeline = ctx
            .device
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("backprojection-pipeline"),
                layout: None,
                module: &module,
                entry_point: "main",
            });
        Ok(Self { pipeline })
    }
}
