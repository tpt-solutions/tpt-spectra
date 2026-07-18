# Contributing to TPT Spectra

Thanks for your interest in contributing! TPT Spectra is an open-source project
maintained by TPT Solutions.

## Getting Started

1. Fork the repository and create a feature branch from `master`.
2. Make your change. Keep commits focused and write clear messages.
3. Ensure the workspace builds and passes checks:
   ```sh
   cargo build --workspace
   cargo test  --workspace
   cargo fmt   --all -- --check
   cargo clippy --workspace -- -D warnings
   ```
4. Open a pull request describing the *why* and the *what*.

## Code Style

- Format with `cargo fmt`.
- Lint with `cargo clippy`; treat warnings as errors in CI.
- Prefer explicit error types via `thiserror`; use `anyhow` at binary boundaries.
- Document public APIs with `///` doc comments.

## Reporting Issues

Use the issue templates (`bug_report`, `feature_request`) when filing issues.
For security vulnerabilities, **do not** open a public issue — email the
maintainers privately instead.

## License

By contributing, you agree that your contributions are dual-licensed under the
MIT and Apache-2.0 licenses used by this project.
