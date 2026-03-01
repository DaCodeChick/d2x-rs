use anyhow::{Context, Result};
use clap::Parser;
use descent_core::{ase::AseFile, converters::model::ModelConverter};
use std::fs;
use std::path::PathBuf;

/// Convert D2X-XL ASE models to glTF/GLB format
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the ASE model file
    #[arg(short, long)]
    ase: PathBuf,

    /// Output GLB file path
    #[arg(short, long)]
    output: PathBuf,

    /// Model name to use in glTF metadata (defaults to input filename)
    #[arg(short, long)]
    name: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine model name
    let model_name = args.name.unwrap_or_else(|| {
        args.ase
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("model")
            .to_string()
    });

    println!("Converting ASE model to GLB...");
    println!("  Input: {}", args.ase.display());
    println!("  Output: {}", args.output.display());
    println!("  Model name: {}", model_name);
    println!();

    // Load ASE file
    println!("Reading ASE file...");
    let ase_data = fs::read_to_string(&args.ase)
        .with_context(|| format!("Failed to read ASE file: {}", args.ase.display()))?;

    // Parse ASE model
    println!("Parsing ASE model...");
    let ase = AseFile::parse(&ase_data)
        .with_context(|| format!("Failed to parse ASE file: {}", args.ase.display()))?;

    println!("  Scene: {}", ase.scene.filename);
    println!("  Objects: {}", ase.geom_objects.len());
    println!("  Materials: {}", ase.materials.len());

    // Count total geometry
    let total_vertices: usize = ase
        .geom_objects
        .iter()
        .map(|obj| obj.mesh.vertices.len())
        .sum();
    let total_faces: usize = ase
        .geom_objects
        .iter()
        .map(|obj| obj.mesh.faces.len())
        .sum();

    println!("  Total vertices: {}", total_vertices);
    println!("  Total faces: {}", total_faces);
    println!();

    // List materials
    if !ase.materials.is_empty() {
        println!("Materials:");
        for (idx, material) in ase.materials.iter().enumerate() {
            println!("  [{}] {}", idx, material.name);
            if let Some(ref map) = material.map_diffuse {
                println!("      Texture: {}", map.bitmap);
            }
        }
        println!();
    }

    // Convert to GLB
    println!("Converting to GLB...");
    let converter = ModelConverter::new();
    let glb = converter
        .ase_to_glb(&ase, &model_name)
        .with_context(|| "Failed to convert ASE to GLB")?;

    // Write output
    println!("Writing GLB file...");
    fs::write(&args.output, &glb)
        .with_context(|| format!("Failed to write output file: {}", args.output.display()))?;

    println!();
    println!("✓ Conversion complete!");
    println!(
        "  Output size: {} bytes ({:.2} KB)",
        glb.len(),
        glb.len() as f64 / 1024.0
    );
    println!("  Saved to: {}", args.output.display());

    Ok(())
}
