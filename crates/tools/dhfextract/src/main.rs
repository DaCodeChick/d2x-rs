//! DHF/HOG archive extraction tool for Descent 1 & 2
//!
//! Extracts files from DHF format archives (.hog, .sow, .mn2)

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use descent_core::DhfArchive;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "dhfextract")]
#[command(about = "Extract files from Descent DHF/HOG archives", long_about = None)]
#[command(version)]
struct Cli {
    /// Path to .hog/.sow/.mn2 archive file
    archive: PathBuf,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// List files in archive
    List {
        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Extract files from archive
    Extract {
        /// Output directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        output: PathBuf,

        /// Extract specific files only
        #[arg(short, long)]
        files: Vec<String>,

        /// Overwrite existing files
        #[arg(short = 'f', long)]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Open archive
    let mut archive = DhfArchive::open(&cli.archive)
        .context(format!("Failed to open archive: {}", cli.archive.display()))?;

    match cli.command {
        Some(Commands::List { verbose }) => list_files(&archive, verbose),
        Some(Commands::Extract {
            output,
            files,
            force,
        }) => extract_files(&mut archive, &output, &files, force),
        None => {
            // Default: list files
            list_files(&archive, false)
        }
    }
}

fn list_files(archive: &DhfArchive, verbose: bool) -> Result<()> {
    let entries: Vec<_> = archive.entries().collect();

    println!("Archive contains {} files\n", entries.len());

    if verbose {
        println!("{:<40} {:>10} {:>12}", "Filename", "Size", "Offset");
        println!("{:-<40} {:->10} {:->12}", "", "", "");

        let mut total_size = 0u64;
        for entry in &entries {
            println!("{:<40} {:>10} {:>12}", entry.name, entry.size, entry.offset);
            total_size += entry.size as u64;
        }

        println!();
        println!(
            "Total: {} files, {} bytes ({:.2} MB)",
            entries.len(),
            total_size,
            total_size as f64 / 1_048_576.0
        );
    } else {
        println!("{:<40} {:>10}", "Filename", "Size");
        println!("{:-<40} {:->10}", "", "");

        let mut total_size = 0u64;
        for entry in &entries {
            println!("{:<40} {:>10}", entry.name, entry.size);
            total_size += entry.size as u64;
        }

        println!();
        println!("Total: {} files, {} bytes", entries.len(), total_size);
    }

    Ok(())
}

fn extract_files(
    archive: &mut DhfArchive,
    output_dir: &Path,
    file_names: &[String],
    force: bool,
) -> Result<()> {
    // Create output directory if it doesn't exist
    if !output_dir.exists() {
        fs::create_dir_all(output_dir).context(format!(
            "Failed to create output directory: {}",
            output_dir.display()
        ))?;
    }

    let files_to_extract: Vec<String> = if file_names.is_empty() {
        // Extract all files
        archive.entries().map(|e| e.name.clone()).collect()
    } else {
        // Extract specific files
        file_names.to_vec()
    };

    println!(
        "Extracting {} file(s) to: {}\n",
        files_to_extract.len(),
        output_dir.display()
    );

    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for filename in &files_to_extract {
        let output_path = output_dir.join(filename);

        // Check if file already exists
        if output_path.exists() && !force {
            println!(
                "Skipping {} (already exists, use -f to overwrite)",
                filename
            );
            skip_count += 1;
            continue;
        }

        match archive.read_file(filename) {
            Ok(data) => match fs::write(&output_path, data) {
                Ok(_) => {
                    println!(
                        "Extracted: {} ({} bytes)",
                        filename,
                        output_path.metadata().map(|m| m.len()).unwrap_or(0)
                    );
                    success_count += 1;
                }
                Err(e) => {
                    eprintln!("Failed to write {}: {}", filename, e);
                    error_count += 1;
                }
            },
            Err(e) => {
                eprintln!("Failed to extract {}: {}", filename, e);
                error_count += 1;
            }
        }
    }

    println!();
    println!(
        "Summary: {} extracted, {} skipped, {} errors",
        success_count, skip_count, error_count
    );

    if error_count > 0 {
        anyhow::bail!("Extraction completed with {} error(s)", error_count);
    }

    Ok(())
}
