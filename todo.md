1: # TPT Spectra — Project Todo
2: 
3: Open-source, GPU-accelerated DICOM reconstruction engine (Rust). Dual-licensed MIT / Apache-2.0, TPT Solutions.
4: 
5: ## Phase 0 — Project Setup & Governance
6: - [x] Initialize git repo
7: - [x] Scaffold Cargo workspace (`Cargo.toml` workspace root)
8: - [x] Create crate layout: `dicom-parser`, `recon-core`, `gpu-backend`, `ai-bridge`, `cli`
9: - [x] Add `LICENSE-MIT` and `LICENSE-APACHE` (dual license, TPT Solutions)
10: - [x] Write README (project overview, architecture summary, build instructions)
11: - [x] Add CONTRIBUTING.md and CODE_OF_CONDUCT.md
12: - [x] Set up CI (build, test, lint/clippy/fmt across all crates)
13: - [x] Add `.gitignore`, issue templates, PR template
14: 
15: ## Phase 1 — DICOM Native Parser (shared foundation)
16: - [x] Design core data model for DICOM tags/datasets/elements
17: - [x] Implement file meta header parsing
18: - [x] Implement transfer syntax handling (implicit/explicit VR, big/little endian)
19: - [x] Implement pixel data extraction (zero-allocation goal)
20: - [x] Handle common compressed transfer syntaxes (JPEG, JPEG2000, RLE) or stub for later
21: - [x] Build test fixtures / sample DICOM file corpus (writer + round-trip)
22: - [x] Unit + fuzz tests for parser robustness
23: 
24: ## Phase 2 — GPU Reconstruction Backend (shared infra)
25: - [x] Choose and abstract GPU backend (wgpu)
26: - [x] Device/adapter/pipeline initialization layer
27: - [x] Define shared voxel/volume data structures (used across all modalities)
28: - [x] Memory management strategy for large volumetric datasets (upload/readback buffers)
29: - [x] Compute shader scaffolding (WGSL/SPIR-V build pipeline)
30: - [x] Benchmarking harness for GPU kernels (CPU bench example; GPU dispatch pending)
31: 
32: ## Phase 3 — Modality Plug-in Modules (parallel skeletons)
33: - [x] Define common `Modality`/reconstruction trait/interface for engine dispatch
34: - [x] CT: filtered back-projection — module stub
35: - [x] CT: filtered back-projection — working implementation
36: - [x] MRI: iterative reconstruction — module stub
37: - [x] MRI: iterative reconstruction — working implementation
38: - [x] Ultrasound: reconstruction — module stub
39: - [x] PET: reconstruction — module stub
40: - [x] Integration tests per modality against known-good reference outputs
41: 
42: ## Phase 4 — AI Diagnostic Bridge
43: - [x] Integrate ONNX Runtime bindings
44: - [x] Build voxel data → model input adapter (tensor shaping/normalization)
45: - [x] Design plug-in model registry (load/register third-party ONNX models)
46: - [x] Example integration: tumor detection model
47: - [x] Example integration: fracture mapping model
48: - [x] Output handling: diagnostic confidence scores, overlays/annotations
49: 
## Phase 5 — Application/CLI & Developer Experience
51: - [x] Build CLI to run parser → reconstruction → AI pipeline end-to-end
52: - [x] Add example/demo datasets and pipeline walkthrough
53: - [x] Document crate architecture and public APIs
54: - [x] Developer setup guide (GPU drivers, build requirements)
55: 
56: ## Phase 6 — Compliance & Verification
57: - [x] Build phantom image test suites
58: - [x] Cross-reference reconstruction output against DCMTK (C++) for pixel-perfect parity (gated harness)
59: - [x] Performance/regression benchmarking suite
60: - [x] FDA 510(k) validation groundwork/documentation
