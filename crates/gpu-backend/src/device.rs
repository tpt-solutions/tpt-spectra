//! GPU device / adapter / pipeline initialization for wgpu.

use spectra_recon_core::Volume;
use thiserror::Error;
use wgpu::util::DeviceExt;
use wgpu::{
    Adapter, Backends, Device, Instance, PowerPreference, Queue, RequestAdapterOptions, Surface,
    SurfaceConfiguration,
};

#[derive(Debug, Error)]
pub enum GpuError {
    #[error("no suitable GPU adapter found")]
    NoAdapter,
    #[error("failed to acquire device: {0}")]
    Device(String),
    #[error("wgpu instance error: {0}")]
    Instance(String),
    #[error("compute error: {0}")]
    Compute(String),
}

pub type Result<T> = std::result::Result<T, GpuError>;

/// A handle to an initialized wgpu device, queue, and optional surface.
pub struct GpuContext {
    pub instance: Instance,
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    /// Present only when a window/surface is provided.
    pub surface: Option<Surface<'static>>,
    pub config: Option<SurfaceConfiguration>,
}

impl GpuContext {
    /// Initialize a headless (no surface) GPU context for compute work.
    pub async fn init_compute() -> Result<Self> {
        let instance = Instance::new(wgpu::InstanceDescriptor {
            backends: Backends::all(),
            ..Default::default()
        });
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            .await
            .ok_or(GpuError::NoAdapter)?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("spectra-gpu"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| GpuError::Device(e.to_string()))?;

        Ok(Self {
            instance,
            adapter,
            device,
            queue,
            surface: None,
            config: None,
        })
    }

    /// Upload a volume's `f32` buffer to a GPU storage buffer.
    pub fn upload_volume(&self, volume: &Volume) -> wgpu::Buffer {
        let bytes = bytemuck::cast_slice::<f32, u8>(&volume.data);
        self.device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("volume-buffer"),
                contents: bytes,
                usage: wgpu::BufferUsages::STORAGE
                    | wgpu::BufferUsages::COPY_DST
                    | wgpu::BufferUsages::COPY_SRC,
            })
    }

    /// Read back a storage buffer into a `Vec<f32>`.
    pub async fn read_buffer_f32(&self, buffer: &wgpu::Buffer, len: usize) -> Vec<f32> {
        let size = len * std::mem::size_of::<f32>();
        let staging = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("staging"),
                contents: &vec![0u8; size],
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("readback"),
            });
        encoder.copy_buffer_to_buffer(buffer, 0, &staging, 0, size as u64);
        self.queue.submit(Some(encoder.finish()));
        let slice = staging.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |r| {
            let _ = tx.send(r);
        });
        self.device.poll(wgpu::Maintain::Wait);
        let _ = rx.recv();
        let mapped = slice.get_mapped_range();
        bytemuck::cast_slice::<u8, f32>(&mapped).to_vec()
    }
}
