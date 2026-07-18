# TPT Spectra

> Open-source, GPU-accelerated DICOM reconstruction engine, written in Rust.

TPT Spectra turns raw medical sensor data (CT, MRI, Ultrasound, PET) into 3D
diagnostic voxel volumes in seconds, then runs third-party AI diagnostic models
directly on the reconstructed data.

It is part of the TPT ecosystem: `tpt-healthcare-nz` stores the FHIR patient
record, while TPT Spectra manages the imaging machines and pushes reconstructed
images and AI confidence scores back into the patient's record.

## License

Dual-licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

at your option. Copyright (c) TPT Solutions.

## Architecture

```text
+-------------------------------------------------------+
|                      spectra-cli                       |
+-------------------+-----------------------------------+
|   dicom-parser    |  Zero-alloc DICOM file parsing     |
|   recon-core      |  Shared volume model + Modality    |
|   gpu-backend     |  wgpu compute (FBP, iterative)     |
|   ai-bridge       |  ONNX Runtime model registry       |
+-------------------------------------------------------+
```

| Crate                | Responsibility                                                  |
|----------------------|----------------------------------------------------------------|
| `dicom-parser`       | DICOM tags, datasets, file meta, transfer syntaxes, pixel data |
| `recon-core`         | Shared volume data structures and the `Modality` trait         |
| `gpu-backend`        | wgpu device/pipeline init and compute-shader reconstruction    |
| `ai-bridge`          | ONNX Runtime bindings, voxel→tensor adapter, model registry    |
| `cli`                | End-to-end `parse → reconstruct → AI` command-line tool        |

## Current Status

Active development. Phases 0–5 are implemented and the workspace builds and
passes `cargo test`, `cargo fmt --check`, and `cargo clippy -- -D warnings`.
See [todo.md](todo.md) for the phased roadmap.

- [x] Cargo workspace + crate layout
- [x] Dual-license + README + governance docs + CI
- [x] DICOM parser: data model, file meta, transfer syntaxes, pixel data, writer
- [x] GPU reconstruction backend (wgpu device/pipeline init + WGSL kernels)
- [x] Modality implementations: CT FBP (working), MRI iterative (working),
      Ultrasound / PET (stubs)
- [x] AI diagnostic bridge (ONNX Runtime, voxel adapter, model registry)
- [x] CLI end-to-end pipeline (`parse` / `reconstruct` / `diagnose` / `run`)
- [x] Verification: phantom tests, DCMTK parity harness (gated), benchmark,
      FDA 510(k) groundwork docs

### Roadmap gaps

- GPU compute path is scaffolded (device init + WGSL) but the FBP compute
  dispatch is not yet wired to a full GPU pipeline run.
- Ultrasound and PET reconstruction are stubbed (interface defined, algorithms
  pending).
- Real ONNX model files (`models/*.onnx`) and full DCMTK pixel-parity are not
  bundled; the parity harness is gated on `SPECTRA_DCMTK_BIN`.

## Building

Requirements:

- Rust 1.74+ (install via [rustup](https://rustup.rs))
- A GPU with Vulkan / Metal / DX12 support (for `gpu-backend`)

```sh
cargo build --workspace
cargo test  --workspace
```

### Running the CLI

```sh
cargo run -p spectra-cli -- parse path/to/study.dcm
```

## Verification & Compliance

Long-term goals (Phase 6) include phantom-image test suites, pixel-perfect
parity against [DCMTK](https://dcmtk.org), and FDA 510(k) validation groundwork.
This software is **not** a certified medical device; do not use it for clinical
diagnosis without appropriate regulatory validation.
