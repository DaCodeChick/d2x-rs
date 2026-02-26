//! Simple MIDI player example demonstrating the MidiPlugin.
//!
//! This example shows how to play MIDI music using the d2x-engine audio system.
//!
//! # Usage
//!
//! You need two files to run this example:
//! 1. A SoundFont file (`.sf2` format) - for instrument samples
//! 2. A MIDI file (`.mid` format) or HMP file (`.hmp` format from Descent)
//!
//! Run with:
//! ```bash
//! cargo run --package d2x-engine --example midi_player -- <soundfont.sf2> <music.mid>
//! ```
//!
//! # Controls
//!
//! - **Space**: Pause/Resume playback
//! - **Escape**: Stop playback and exit

use bevy::prelude::*;
use d2x_engine::audio::{MidiPlaybackEvent, MidiPlugin};
use descent_core::HmpFile;
use std::env;
use std::fs;
use std::process;
use tracing::{error, info};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <soundfont.sf2> <music.mid|music.hmp>", args[0]);
        eprintln!();
        eprintln!("Example:");
        eprintln!(
            "  cargo run --package d2x-engine --example midi_player -- soundfont.sf2 game01.hmp"
        );
        process::exit(1);
    }

    let soundfont_path = &args[1];
    let music_path = &args[2];

    // Verify files exist
    if !std::path::Path::new(soundfont_path).exists() {
        eprintln!("Error: SoundFont file not found: {}", soundfont_path);
        process::exit(1);
    }

    if !std::path::Path::new(music_path).exists() {
        eprintln!("Error: Music file not found: {}", music_path);
        process::exit(1);
    }

    println!("Starting MIDI player...");
    println!("SoundFont: {}", soundfont_path);
    println!("Music: {}", music_path);
    println!();
    println!("Controls:");
    println!("  Space - Pause/Resume");
    println!("  Escape - Stop and exit");
    println!();

    App::new()
        .add_plugins((DefaultPlugins, MidiPlugin::new(soundfont_path)))
        .insert_resource(MusicFile(music_path.clone()))
        .add_systems(Startup, setup)
        .add_systems(Update, handle_input)
        .run();
}

#[derive(Resource)]
struct MusicFile(String);

#[derive(Resource)]
struct PlaybackState {
    playing: bool,
    paused: bool,
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            playing: false,
            paused: false,
        }
    }
}

fn setup(mut commands: Commands, music_file: Res<MusicFile>) {
    // Read music file
    let music_data = match fs::read(&music_file.0) {
        Ok(data) => data,
        Err(e) => {
            error!("Failed to read music file: {}", e);
            process::exit(1);
        }
    };

    // Convert to MIDI if needed
    let midi_data = if music_file.0.to_lowercase().ends_with(".hmp") {
        info!("Converting HMP to MIDI...");
        match HmpFile::parse(&music_data) {
            Ok(hmp) => match hmp.to_midi() {
                Ok(midi) => midi,
                Err(e) => {
                    error!("Failed to convert HMP to MIDI: {}", e);
                    process::exit(1);
                }
            },
            Err(e) => {
                error!("Failed to parse HMP file: {}", e);
                process::exit(1);
            }
        }
    } else {
        music_data
    };

    // Initialize playback state
    commands.init_resource::<PlaybackState>();

    // Start playback
    commands.trigger(MidiPlaybackEvent::Play {
        midi_data,
        looping: true,
    });

    // Mark as playing
    commands.insert_resource(PlaybackState {
        playing: true,
        paused: false,
    });

    info!("MIDI playback started");
}

fn handle_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<PlaybackState>,
) {
    // Escape - Stop and exit
    if keyboard.just_pressed(KeyCode::Escape) {
        info!("Stopping playback...");
        commands.trigger(MidiPlaybackEvent::Stop);
        // In a real application, you'd trigger AppExit here
        // For now, just stop playback
        return;
    }

    // Space - Pause/Resume
    if keyboard.just_pressed(KeyCode::Space) {
        if state.playing {
            if state.paused {
                info!("Resuming playback...");
                commands.trigger(MidiPlaybackEvent::Resume);
                state.paused = false;
            } else {
                info!("Pausing playback...");
                commands.trigger(MidiPlaybackEvent::Pause);
                state.paused = true;
            }
        }
    }
}
