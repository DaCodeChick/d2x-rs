//! Audio system for D2X Engine with MIDI synthesis support.
//!
//! This module provides Bevy integration for playing Descent music and sound effects:
//! - **MIDI playback** via rustysynth software synthesizer
//! - **HMP → MIDI conversion** using descent-core
//! - **PCM sound effects** from Descent PIG files
//!
//! # Examples
//!
//! ## Playing HMP music
//!
//! ```no_run
//! use bevy::prelude::*;
//! use d2x_engine::audio::{MidiPlugin, MidiPlaybackEvent};
//! use descent_core::HmpFile;
//!
//! fn play_music(mut commands: Commands) {
//!     // Load HMP file data
//!     let hmp_data = std::fs::read("game01.hmp").unwrap();
//!     let hmp = HmpFile::parse(&hmp_data).unwrap();
//!     
//!     // Convert to MIDI and trigger playback event
//!     let midi_data = hmp.to_midi().unwrap();
//!     commands.trigger(MidiPlaybackEvent::Play {
//!         midi_data,
//!         looping: true,
//!     });
//! }
//! ```

use bevy::ecs::event::Event;
use bevy::ecs::observer::On;
use bevy::prelude::*;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::sync::{Arc, Mutex};
use tracing::{error, info};

/// MIDI playback plugin for Bevy.
///
/// Provides software MIDI synthesis using rustysynth and integration with
/// Bevy's audio system. Requires a SoundFont file for instrument samples.
///
/// # Usage
///
/// Add to your Bevy app:
///
/// ```no_run
/// use bevy::prelude::*;
/// use d2x_engine::audio::MidiPlugin;
///
/// App::new()
///     .add_plugins(MidiPlugin {
///         soundfont_path: "soundfont.sf2".to_string(),
///     })
///     .run();
/// ```
pub struct MidiPlugin {
    /// Path to SoundFont (.sf2) file for synthesis.
    pub soundfont_path: String,
}

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        // Store soundfont path as a resource
        app.insert_resource(SoundfontPath(self.soundfont_path.clone()))
            .init_resource::<MidiState>()
            // Add single observer for all MIDI playback events
            .add_observer(handle_midi_events);
    }
}

/// Path to the SoundFont file.
#[derive(Resource, Clone)]
struct SoundfontPath(String);

/// MIDI playback state.
#[derive(Resource, Default)]
pub struct MidiState {
    /// Current MIDI sequencer (if any).
    /// The sequencer owns the synthesizer internally.
    sequencer: Option<Arc<Mutex<MidiFileSequencer>>>,
    /// Whether playback is looping.
    looping: bool,
}

/// Events for controlling MIDI playback.
#[derive(Event, Clone)]
pub enum MidiPlaybackEvent {
    /// Start playing MIDI data.
    Play {
        /// Standard MIDI file data.
        midi_data: Vec<u8>,
        /// Whether to loop playback.
        looping: bool,
    },
    /// Stop current playback.
    Stop,
    /// Pause playback.
    Pause,
    /// Resume playback.
    Resume,
}

/// Observer for all MIDI playback events.
fn handle_midi_events(
    event: On<MidiPlaybackEvent>,
    mut state: ResMut<MidiState>,
    soundfont_path: Res<SoundfontPath>,
) {
    match event.event() {
        MidiPlaybackEvent::Play { midi_data, looping } => {
            if let Err(e) = start_midi_playback(&mut state, &soundfont_path.0, midi_data, looping) {
                error!("Failed to start MIDI playback: {}", e);
            }
        }
        MidiPlaybackEvent::Stop => {
            state.sequencer = None;
            info!("MIDI playback stopped");
        }
        MidiPlaybackEvent::Pause => {
            // TODO: Implement pause functionality
            info!("MIDI pause not yet implemented");
        }
        MidiPlaybackEvent::Resume => {
            // TODO: Implement resume functionality
            info!("MIDI resume not yet implemented");
        }
    }
}

/// Start MIDI playback with the given data.
fn start_midi_playback(
    state: &mut MidiState,
    soundfont_path: &str,
    midi_data: &Vec<u8>,
    looping: &bool,
) -> Result<(), String> {
    // Load SoundFont
    let mut sf_file = std::fs::File::open(soundfont_path)
        .map_err(|e| format!("Failed to open SoundFont file: {}", e))?;
    let soundfont = Arc::new(
        SoundFont::new(&mut sf_file).map_err(|e| format!("Failed to load SoundFont: {}", e))?,
    );

    // Parse MIDI file
    let midi_file = Arc::new(
        MidiFile::new(&mut std::io::Cursor::new(midi_data))
            .map_err(|e| format!("Failed to parse MIDI file: {}", e))?,
    );

    // Create synthesizer settings (44.1 kHz, stereo)
    let settings = SynthesizerSettings::new(44100);

    // Create synthesizer
    let synthesizer = Synthesizer::new(&soundfont, &settings)
        .map_err(|e| format!("Failed to create synthesizer: {}", e))?;

    // Create sequencer (takes ownership of synthesizer)
    let mut sequencer = MidiFileSequencer::new(synthesizer);

    // Load MIDI into sequencer
    sequencer.play(&midi_file, *looping);

    // Store state (wrap in Arc<Mutex<>> for thread safety)
    state.sequencer = Some(Arc::new(Mutex::new(sequencer)));
    state.looping = *looping;

    info!("Started MIDI playback (looping: {})", looping);
    Ok(())
}

/// Resource for managing audio samples.
#[derive(Resource)]
pub struct AudioSampleBuffer {
    /// Pre-rendered audio samples (for streaming to Bevy audio).
    pub buffer: Vec<f32>,
    /// Sample rate.
    pub sample_rate: u32,
}

impl Default for AudioSampleBuffer {
    fn default() -> Self {
        Self {
            buffer: Vec::new(),
            sample_rate: 44100,
        }
    }
}

/// Render MIDI to audio samples.
///
/// This system should be called regularly to fill the audio buffer
/// with synthesized samples from the MIDI sequencer.
pub fn render_midi_samples(state: ResMut<MidiState>, mut buffer: ResMut<AudioSampleBuffer>) {
    if let Some(ref sequencer) = state.sequencer {
        let samples_needed = 4096; // Render in 4K chunks
        let mut left = vec![0.0; samples_needed];
        let mut right = vec![0.0; samples_needed];

        // Render samples
        if let Ok(mut seq) = sequencer.lock() {
            seq.render(&mut left, &mut right);
        }

        // Interleave stereo samples
        buffer.buffer.clear();
        buffer.buffer.reserve(samples_needed * 2);
        for i in 0..samples_needed {
            buffer.buffer.push(left[i]);
            buffer.buffer.push(right[i]);
        }
    }
}

/// Component for tagging entities with PCM sound effects from Descent.
#[derive(Component)]
pub struct DescentSound {
    /// Sound name (from PIG file).
    pub name: String,
    /// PCM samples (8-bit unsigned, converted to f32).
    pub samples: Vec<f32>,
    /// Sample rate (typically 11025 or 22050 Hz).
    pub sample_rate: u32,
}

impl DescentSound {
    /// Create from descent-core SoundData.
    pub fn from_sound_data(sound: &descent_core::SoundData) -> Self {
        Self {
            name: sound.header.name.clone(),
            samples: sound.to_f32_samples(),
            sample_rate: sound.sample_rate_hint(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_midi_plugin_creation() {
        let plugin = MidiPlugin {
            soundfont_path: "test.sf2".to_string(),
        };
        assert_eq!(plugin.soundfont_path, "test.sf2");
    }

    #[test]
    fn test_midi_state_default() {
        let state = MidiState::default();
        assert!(state.sequencer.is_none());
        assert!(!state.looping);
    }

    #[test]
    fn test_audio_sample_buffer_default() {
        let buffer = AudioSampleBuffer::default();
        assert_eq!(buffer.sample_rate, 44100);
        assert!(buffer.buffer.is_empty());
    }
}
