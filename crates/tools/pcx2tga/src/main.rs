//! PCX to TGA image converter for Descent briefing screens and images
//!
//! Converts PCX images to TGA format, commonly used in Descent 1 & 2 for
//! briefing screens, ending screens, and other full-screen images.

use anyhow::{Context, Result};
use clap::Parser;
use descent_core::PcxImage;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "pcx2tga")]
#[command(about = "Convert PCX images to TGA format", long_about = None)]
#[command(version)]
struct Cli {
    /// Input PCX file(s)
    input: Vec<PathBuf>,

    /// Output directory (default: same as input)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output file name (only valid for single input)
    #[arg(short = 'O', long)]
    output_file: Option<PathBuf>,

    /// Overwrite existing files
    #[arg(short, long)]
    force: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.input.is_empty() {
        anyhow::bail!("No input files specified");
    }

    // Validate output_file is only used with single input
    if cli.output_file.is_some() && cli.input.len() > 1 {
        anyhow::bail!("--output-file can only be used with a single input file");
    }

    let mut success_count = 0;
    let mut error_count = 0;

    for input_path in &cli.input {
        match convert_pcx(input_path, &cli) {
            Ok(output_path) => {
                if cli.verbose {
                    println!(
                        "✓ Converted: {} -> {}",
                        input_path.display(),
                        output_path.display()
                    );
                }
                success_count += 1;
            }
            Err(e) => {
                eprintln!("✗ Failed to convert {}: {}", input_path.display(), e);
                error_count += 1;
            }
        }
    }

    if cli.input.len() > 1 || cli.verbose {
        println!();
        println!(
            "Summary: {} converted, {} errors",
            success_count, error_count
        );
    }

    if error_count > 0 {
        anyhow::bail!("Conversion completed with {} error(s)", error_count);
    }

    Ok(())
}

fn convert_pcx(input_path: &Path, cli: &Cli) -> Result<PathBuf> {
    // Read PCX file
    let pcx_data = fs::read(input_path)
        .context(format!("Failed to read PCX file: {}", input_path.display()))?;

    // Parse PCX
    let pcx = PcxImage::parse(&pcx_data)
        .context(format!("Failed to parse PCX: {}", input_path.display()))?;

    if cli.verbose {
        println!(
            "  {}x{}, {} bpp, {}",
            pcx.width(),
            pcx.height(),
            pcx.bits_per_pixel(),
            if pcx.is_indexed() { "indexed" } else { "RGB" }
        );
    }

    // Convert to TGA
    let tga_data = pcx.to_tga().context(format!(
        "Failed to convert to TGA: {}",
        input_path.display()
    ))?;

    // Determine output path
    let output_path = if let Some(ref output_file) = cli.output_file {
        output_file.clone()
    } else {
        let output_dir = cli
            .output
            .as_ref()
            .map(|p| p.as_path())
            .unwrap_or_else(|| input_path.parent().unwrap_or_else(|| Path::new(".")));

        let file_stem = input_path.file_stem().context("Invalid input filename")?;

        output_dir.join(format!("{}.tga", file_stem.to_string_lossy()))
    };

    // Check if output exists
    if output_path.exists() && !cli.force {
        anyhow::bail!(
            "Output file already exists: {} (use --force to overwrite)",
            output_path.display()
        );
    }

    // Create output directory if needed
    if let Some(parent) = output_path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).context(format!(
                "Failed to create output directory: {}",
                parent.display()
            ))?;
        }
    }

    // Write TGA file
    fs::write(&output_path, tga_data).context(format!(
        "Failed to write TGA file: {}",
        output_path.display()
    ))?;

    Ok(output_path)
}
