//! TPT Spectra CLI entry point.
//!
//! Runs the end-to-end pipeline: parse DICOM -> reconstruct -> AI diagnostics.

mod pipeline;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::info;
use tracing_subscriber::EnvFilter;

use pipeline::{load_dicom, pixels_to_volume, reconstruct_from_volume, run_diagnostic};

#[derive(Parser)]
#[command(
    name = "spectra",
    version,
    about = "TPT Spectra DICOM reconstruction engine"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Parse a DICOM file and print a summary.
    Parse {
        /// Path to the input DICOM file.
        path: std::path::PathBuf,
    },
    /// Reconstruct a volume from a DICOM image (treats slices as CT projections).
    Reconstruct {
        /// Path to the input DICOM file.
        path: std::path::PathBuf,
    },
    /// Run an example diagnostic model over a reconstructed volume.
    Diagnose {
        /// Path to the input DICOM file.
        path: std::path::PathBuf,
        /// Model id from the registry (e.g. tumor-detection-v1).
        #[arg(default_value = "tumor-detection-v1")]
        model: String,
    },
    /// Run the full parse -> reconstruct -> diagnose pipeline.
    Run {
        /// Path to the input DICOM file.
        path: std::path::PathBuf,
        /// Model id from the registry.
        #[arg(default_value = "tumor-detection-v1")]
        model: String,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let cli = Cli::parse();
    match cli.command {
        Command::Parse { path } => cmd_parse(&path),
        Command::Reconstruct { path } => cmd_reconstruct(&path),
        Command::Diagnose { path, model } => cmd_diagnose(&path, &model),
        Command::Run { path, model } => {
            cmd_parse(&path)?;
            cmd_reconstruct(&path)?;
            cmd_diagnose(&path, &model)
        }
    }
}

fn cmd_parse(path: &std::path::Path) -> Result<()> {
    let file = load_dicom(path)?;
    println!("Transfer syntax: {}", file.transfer_syntax);
    println!("Meta elements: {}", file.meta.elements.len());
    println!("Dataset elements: {}", file.dataset.elements.len());
    if let Ok(px) = spectra_dicom_parser::extract_pixels(&file.dataset, file.transfer_syntax) {
        println!(
            "Pixel data: {}x{} ({} bits, {} samples)",
            px.rows, px.columns, px.bits_allocated, px.samples_per_pixel
        );
    }
    Ok(())
}

fn cmd_reconstruct(path: &std::path::Path) -> Result<()> {
    let file = load_dicom(path)?;
    let px = spectra_dicom_parser::extract_pixels(&file.dataset, file.transfer_syntax)
        .context("extracting pixel data")?;
    let vol = pixels_to_volume(&px);
    let recon = reconstruct_from_volume(&vol)?;
    info!(
        "reconstructed volume {}x{}x{}",
        recon.dims[0], recon.dims[1], recon.dims[2]
    );
    println!(
        "Reconstructed volume: {}x{}x{} voxels",
        recon.dims[0], recon.dims[1], recon.dims[2]
    );
    Ok(())
}

fn cmd_diagnose(path: &std::path::Path, model: &str) -> Result<()> {
    let file = load_dicom(path)?;
    let px = spectra_dicom_parser::extract_pixels(&file.dataset, file.transfer_syntax)
        .context("extracting pixel data")?;
    let vol = pixels_to_volume(&px);
    let recon = reconstruct_from_volume(&vol)?;
    let annotations = run_diagnostic(&recon, model)?;
    println!("Diagnostic annotations:");
    for a in annotations {
        println!("  - {a}");
    }
    Ok(())
}
