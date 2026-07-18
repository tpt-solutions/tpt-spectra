# Developer Setup Guide

## Prerequisites

- **Rust** 1.74+ — install via [rustup](https://rustup.rs).
- **Git** for cloning and contributing.
- A **GPU with Vulkan / Metal / DX12** support is required *only* for the
  `spectra-gpu-backend` compute path. All CPU reconstruction paths and tests
  run without a GPU. CI runners without a GPU skip GPU tests automatically.

## Building

```sh
git clone https://github.com/TPT-Solutions/tpt-spectra
cd tpt-spectra
cargo build --workspace
```

## Testing

```sh
cargo test --workspace
```

- GPU tests are marked `#[ignore]` and only run when `SPECTRA_RUN_GPU=1`.
- The DCMTK parity smoke test is ignored unless `SPECTRA_DCMTK_BIN` points at a
  DCMTK binary directory.

## Formatting & linting

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

CI enforces both.

## Running the CLI

```sh
# Parse a DICOM file
cargo run -p spectra-cli -- parse path/to/study.dcm

# Reconstruct a volume
cargo run -p spectra-cli -- reconstruct path/to/study.dcm

# End-to-end parse -> reconstruct -> diagnose
cargo run -p spectra-cli -- run path/to/study.dcm
```

## Generating a demo phantom

```sh
cargo run -p spectra-cli --example phantom -- /tmp/phantom.dcm
```

## Running the benchmark

```sh
cargo run -p spectra-recon-core --example bench --release
```
