# Crate Architecture

TPT Spectra is a Cargo workspace of five crates. Dependencies flow strictly
downward; no crate depends on a sibling "above" it.

```
spectra-cli
   ├─ spectra-dicom-parser
   ├─ spectra-recon-core
   │      └─ (used by gpu-backend + ai-bridge)
   ├─ spectra-gpu-backend ──> spectra-recon-core
   └─ spectra-ai-bridge  ──> spectra-recon-core
```

## `spectra-dicom-parser`

Zero-allocation DICOM parsing.

- `model` — `Tag`, `VR`, `Element`, `Dataset` data model.
- `transfer_syntax` — implicit/explicit VR, endianness, compression, known UIDs.
- `parser` — file meta header + dataset parsing (Explicit/Implicit, LE/BE).
- `pixel` — uncompressed pixel-data extraction and geometry.
- `writer` — minimal Explicit VR LE serializer (fixtures, phantoms).

Public entry points: `parse_file`, `extract_pixels`, `write_explicit_le`.

## `spectra-recon-core`

Shared reconstruction model and dispatch.

- `volume` — `Volume` (row-major `f32` buffer, multi-sample aware).
- `geometry` — `Projection`, `CtGeometry`, `MriGeometry`.
- `modality` — `Reconstructor` trait, `Modality` enum, `reconstruct_ct` /
  `reconstruct_mri` dispatch. Contains `ct` (filtered back-projection),
  `mri` (iterative IFFT + ISTA), `ultrasound` / `pet` (stubs).

## `spectra-gpu-backend`

GPU compute via `wgpu`.

- `device` — `GpuContext` (adapter/device/queue init, upload/readback).
- `shader` — WGSL kernel sources + `ComputeKernel` builder. The back-projection
  kernel mirrors the CPU FBP so the two paths can be cross-validated.

## `spectra-ai-bridge`

AI diagnostic bridge via ONNX Runtime (`ort`).

- `adapter` — `Volume` → NCHW/NCDHW tensor, normalization.
- `registry` — `ModelRegistry`, `ModelDescriptor`, example tumor/fracture models.
- `inference` — `DiagnosticModel` (loads `.onnx`, runs inference).
- `output` — `Diagnostic` scores, annotation strings.

## `spectra-cli`

End-to-end command-line tool wiring parse → reconstruct → diagnose, plus the
`phantom` demo example and the `bench` micro-benchmark.
