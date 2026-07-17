# TPT Spectra — Project Todo

Open-source, GPU-accelerated DICOM reconstruction engine (Rust). Dual-licensed MIT / Apache-2.0, TPT Solutions.

## Phase 0 — Project Setup & Governance
- [ ] Initialize git repo
- [ ] Scaffold Cargo workspace (`Cargo.toml` workspace root)
- [ ] Create crate layout: `dicom-parser`, `recon-core`, `gpu-backend`, `ai-bridge`, `cli`
- [ ] Add `LICENSE-MIT` and `LICENSE-APACHE` (dual license, TPT Solutions)
- [ ] Write README (project overview, architecture summary, build instructions)
- [ ] Add CONTRIBUTING.md and CODE_OF_CONDUCT.md
- [ ] Set up CI (build, test, lint/clippy/fmt across all crates)
- [ ] Add `.gitignore`, issue templates, PR template

## Phase 1 — DICOM Native Parser (shared foundation)
- [ ] Design core data model for DICOM tags/datasets/elements
- [ ] Implement file meta header parsing
- [ ] Implement transfer syntax handling (implicit/explicit VR, big/little endian)
- [ ] Implement pixel data extraction (zero-allocation goal)
- [ ] Handle common compressed transfer syntaxes (JPEG, JPEG2000, RLE) or stub for later
- [ ] Build test fixtures / sample DICOM file corpus
- [ ] Unit + fuzz tests for parser robustness

## Phase 2 — GPU Reconstruction Backend (shared infra)
- [ ] Choose and abstract GPU backend (wgpu vs. raw Vulkan)
- [ ] Device/adapter/pipeline initialization layer
- [ ] Define shared voxel/volume data structures (used across all modalities)
- [ ] Memory management strategy for large volumetric datasets
- [ ] Compute shader scaffolding (WGSL/SPIR-V build pipeline)
- [ ] Benchmarking harness for GPU kernels

## Phase 3 — Modality Plug-in Modules (parallel skeletons)
- [ ] Define common `Modality`/reconstruction trait/interface for engine dispatch
- [ ] CT: filtered back-projection — module stub
- [ ] CT: filtered back-projection — working implementation
- [ ] MRI: iterative reconstruction — module stub
- [ ] MRI: iterative reconstruction — working implementation
- [ ] Ultrasound: reconstruction — module stub
- [ ] PET: reconstruction — module stub
- [ ] Integration tests per modality against known-good reference outputs

## Phase 4 — AI Diagnostic Bridge
- [ ] Integrate ONNX Runtime bindings
- [ ] Build voxel data → model input adapter (tensor shaping/normalization)
- [ ] Design plug-in model registry (load/register third-party ONNX models)
- [ ] Example integration: tumor detection model
- [ ] Example integration: fracture mapping model
- [ ] Output handling: diagnostic confidence scores, overlays/annotations

## Phase 5 — Application/CLI & Developer Experience
- [ ] Build CLI to run parser → reconstruction → AI pipeline end-to-end
- [ ] Add example/demo datasets and pipeline walkthrough
- [ ] Document crate architecture and public APIs
- [ ] Developer setup guide (GPU drivers, build requirements)

## Phase 6 — Compliance & Verification
- [ ] Build phantom image test suites
- [ ] Cross-reference reconstruction output against DCMTK (C++) for pixel-perfect parity
- [ ] Performance/regression benchmarking suite
- [ ] FDA 510(k) validation groundwork/documentation
