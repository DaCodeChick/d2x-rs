//! Example program to parse and display HXM file information

use descent_core::hxm::HxmFile;
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <hxm_file>", args[0]);
        eprintln!();
        eprintln!("Example: cargo run --example parse_hxm /tmp/hxm-test/CAMBOT.HXM");
        process::exit(1);
    }

    let filename = &args[1];

    // Read the file
    let data = match fs::read(filename) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Error reading file '{}': {}", filename, e);
            process::exit(1);
        }
    };

    println!("Parsing HXM file: {}", filename);
    println!("File size: {} bytes", data.len());
    println!();

    // Parse the HXM file
    let hxm = match HxmFile::parse(&data) {
        Ok(hxm) => hxm,
        Err(e) => {
            eprintln!("Error parsing HXM file: {}", e);
            process::exit(1);
        }
    };

    println!("HXM Information:");
    println!("  Version: {}", hxm.version());
    println!("  Robot count: {}", hxm.robot_count());
    println!("  Extra data size: {} bytes", hxm.extra_data().len());
    println!();

    // Display information about each custom robot
    println!("Custom Robots:");
    for (i, (index, robot_data)) in hxm.custom_robots().enumerate() {
        println!("  Robot #{}: Index {}", i + 1, index);
        println!("    Robot info size: {} bytes", robot_data.len());
    }

    if hxm.extra_data().len() > 0 {
        println!();
        println!("Extra data contains custom models, joints, weapons, and animations");
        println!(
            "  First 16 bytes: {:02x?}",
            &hxm.extra_data()[..16.min(hxm.extra_data().len())]
        );
    }

    println!();
    println!("✓ Successfully parsed HXM file!");
}
