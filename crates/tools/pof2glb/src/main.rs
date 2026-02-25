use anyhow::{Context, Result};
use clap::Parser;
use descent_core::{
    converters::model::{ModelConverter, TextureProvider},
    ham::HamFile,
    palette::Palette,
    pig::PigFile,
    pof::PofParser,
};
use std::fs;
use std::path::PathBuf;

/// Convert Descent POF models to glTF/GLB format with texture support
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the POF model file
    #[arg(short, long)]
    pof: PathBuf,

    /// Path to the HAM file (contains texture metadata)
    #[arg(long)]
    ham: PathBuf,

    /// Path to the PIG file (contains texture data)
    #[arg(long)]
    pig: PathBuf,

    /// Path to the palette file (.256 or .pal)
    #[arg(long)]
    palette: PathBuf,

    /// Output GLB file path
    #[arg(short, long)]
    output: PathBuf,

    /// Model name to use in glTF metadata (defaults to input filename)
    #[arg(short, long)]
    name: Option<String>,

    /// Parse POF with embedded header (from HAM extraction)
    #[arg(long, default_value_t = false)]
    with_header: bool,

    /// Treat PIG file as Descent 1 format
    #[arg(long, default_value_t = false)]
    d1: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Determine model name
    let model_name = args.name.unwrap_or_else(|| {
        args.pof
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("model")
            .to_string()
    });

    println!("Loading Descent data files...");

    // Load palette
    println!("  Reading palette: {}", args.palette.display());
    let palette_data = fs::read(&args.palette)
        .with_context(|| format!("Failed to read palette file: {}", args.palette.display()))?;
    let palette = Palette::parse(&palette_data)
        .with_context(|| format!("Failed to parse palette file: {}", args.palette.display()))?;

    // Load PIG file
    println!("  Reading PIG: {}", args.pig.display());
    let pig_data = fs::read(&args.pig)
        .with_context(|| format!("Failed to read PIG file: {}", args.pig.display()))?;
    let pig = PigFile::parse(pig_data, args.d1)
        .with_context(|| format!("Failed to parse PIG file: {}", args.pig.display()))?;

    // Load HAM file
    println!("  Reading HAM: {}", args.ham.display());
    let ham_data = fs::read(&args.ham)
        .with_context(|| format!("Failed to read HAM file: {}", args.ham.display()))?;
    let ham = HamFile::parse(&ham_data)
        .with_context(|| format!("Failed to parse HAM file: {}", args.ham.display()))?;

    // Load POF file
    println!("  Reading POF: {}", args.pof.display());
    let pof_data = fs::read(&args.pof)
        .with_context(|| format!("Failed to read POF file: {}", args.pof.display()))?;

    // Parse POF model
    println!("Parsing POF model...");
    let pof_model = if args.with_header {
        PofParser::parse_with_header(&pof_data)
    } else {
        PofParser::parse(&pof_data)
    }
    .with_context(|| format!("Failed to parse POF file: {}", args.pof.display()))?;

    println!("  Model info:");
    println!("    Vertices: {}", pof_model.vertices.len());
    println!("    Polygons: {}", pof_model.polygons.len());
    println!("    Textures referenced: {}", pof_model.n_textures);
    println!("    First texture slot: {}", pof_model.first_texture);

    // Create texture provider
    let texture_provider = TextureProvider::new(pig, palette.clone(), ham);

    // Convert to GLB
    println!("Converting POF to GLB...");
    let converter = ModelConverter::new();
    let glb_data = converter
        .pof_to_glb(&pof_model, &model_name, Some(&texture_provider))
        .context("Failed to convert POF to GLB")?;

    // Write output file
    println!("Writing GLB file: {}", args.output.display());
    fs::write(&args.output, glb_data)
        .with_context(|| format!("Failed to write output file: {}", args.output.display()))?;

    println!("✓ Conversion complete!");
    println!(
        "  Output: {} ({} bytes)",
        args.output.display(),
        fs::metadata(&args.output)?.len()
    );

    Ok(())
}
