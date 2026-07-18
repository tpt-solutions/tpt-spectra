# FDA 510(k) Validation Groundwork

> **Status: groundwork / documentation only.** TPT Spectra is **not** a
> certified medical device. The contents of this document describe the
> validation strategy that must be completed before any clinical use. They are
> not a substitute for regulatory review.

## Intended use (draft)

TPT Spectra is intended as a research and secondary-capture reconstruction
engine that converts raw medical imaging acquisitions (CT, MRI, Ultrasound,
PET) into volumetric datasets and runs third-party AI diagnostic models. It is
**not** intended to be the primary means of diagnosis.

## Verification building blocks (present in the repo)

1. **Phantom image test suite** — `crates/recon-core/tests/phantom.rs`
   reconstructs synthetic phantoms (e.g. Shepp-Logan-style ellipses) and asserts
   known-good structural properties (finite values, energy concentration,
   central symmetry).
2. **Unit + integration tests** — parser round-trips, modality integration
   tests, AI adapter/registry tests. Run via `cargo test --workspace`.
3. **Cross-reference against DCMTK** — `crates/dicom-parser/tests/dcmtk_parity.rs`
   is a gated harness (`SPECTRA_DCMTK_BIN`) that verifies spectra-written DICOM
   is parseable by the reference DCMTK (C++) toolchain. The full pixel-parity
   comparison is wired here once reference reconstructions are available.
4. **Performance / regression benchmarking** — `crates/recon-core/examples/bench.rs`
   times the CT FBP kernel. Promote to a tracked CI benchmark to catch
   performance regressions.

## Validation plan (to be executed)

| Activity | Owner | Artifact |
|----------|-------|----------|
| Define acceptance criteria per modality | Clinical + Eng | Validation plan |
| Phantom gold-standard reconstructions | Physics | Reference volumes |
| Pixel-perfect parity vs DCMTK | Eng | Parity report |
| AI model performance characterization | Data Science | ROC / calibration reports |
| Hazard analysis (FMEA) | RA/QA | Risk file |
| Software verification & validation (V&V) | QA | V&V protocols + results |
| Configuration management & traceability | RA/QA | Trace matrix |

## Traceability

Each requirement should map to a test in this repository. Tag tests with the
relevant requirement ID (e.g. `REQ-CT-001`) as the suite matures so the trace
matrix can be generated automatically.

## Labeling & limitations

Any deployment built from this code must display prominent non-clinical-use
labeling until 510(k) clearance is obtained, and must not be represented as a
substitute for a certified diagnostic system.
