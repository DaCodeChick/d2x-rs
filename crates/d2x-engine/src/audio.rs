//! Audio system for D2X Engine with MIDI synthesis support.
//!
//! This module provides Bevy integration for playing Descent music and sound effects:
//! - **MIDI playback** via rustysynth software synthesizer with streaming audio output
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
//! fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
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
//!
//! App::new()
//!     .add_plugins((DefaultPlugins, MidiPlugin::new("soundfont.sf2")))
//!     .add_systems(Startup, setup)
//!     .run();
//! ```

use bevy::asset::{Asset, Assets};
use bevy::audio::AudioPlugin as BevyAudioPlugin;
use bevy::audio::{AddAudioSource, AudioPlayer, Decodable, PlaybackSettings, Source};
use bevy::ecs::event::Event;
use bevy::ecs::observer::On;
use bevy::prelude::*;
use bevy::reflect::TypePath;
use rustysynth::{MidiFile, MidiFileSequencer, SoundFont, Synthesizer, SynthesizerSettings};
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tracing::{error, info, warn};

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
///     .add_plugins(MidiPlugin::new("soundfont.sf2"))
///     .run();
/// ```
pub struct MidiPlugin {
    /// Path to SoundFont (.sf2) file for synthesis.
    soundfont_path: String,
}

impl MidiPlugin {
    /// Create a new MIDI plugin with the given SoundFont path.
    pub fn new(soundfont_path: impl Into<String>) -> Self {
        Self {
            soundfont_path: soundfont_path.into(),
        }
    }
}

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        // Ensure Bevy's audio plugin is present
        if !app.is_plugin_added::<BevyAudioPlugin>() {
            warn!("MidiPlugin requires AudioPlugin to be added first");
        }

        // Register our custom audio source type
        app.add_audio_source::<MidiAudioSource>();

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

/// Streaming MIDI audio source for Bevy's audio system.
///
/// This wraps a rustysynth MIDI sequencer and implements Bevy's `Decodable` trait
/// to stream synthesized audio samples on-demand.
#[derive(Asset, TypePath, Clone)]
pub struct MidiAudioSource {
    /// Shared MIDI sequencer (wrapped for thread-safe access).
    sequencer: Arc<Mutex<MidiFileSequencer>>,
    /// Current sample position in the stream.
    position: Arc<Mutex<usize>>,
    /// Whether playback is paused.
    paused: Arc<Mutex<bool>>,
    /// Sample rate (always 44100 Hz).
    sample_rate: u32,
}

impl MidiAudioSource {
    /// Create from MIDI data and a loaded SoundFont.
    pub fn new(soundfont: Arc<SoundFont>, midi_data: &[u8], looping: bool) -> Result<Self, String> {
        // Parse MIDI file
        let midi_file = Arc::new(
            MidiFile::new(&mut Cursor::new(midi_data))
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
        sequencer.play(&midi_file, looping);

        Ok(Self {
            sequencer: Arc::new(Mutex::new(sequencer)),
            position: Arc::new(Mutex::new(0)),
            paused: Arc::new(Mutex::new(false)),
            sample_rate: 44100,
        })
    }

    /// Set pause state.
    pub fn set_paused(&self, paused: bool) {
        if let Ok(mut p) = self.paused.lock() {
            *p = paused;
        }
    }

    /// Check if paused.
    pub fn is_paused(&self) -> bool {
        self.paused.lock().map(|p| *p).unwrap_or(false)
    }
}

/// Iterator over MIDI audio samples.
///
/// This generates audio samples on-demand by calling the MIDI synthesizer.
pub struct MidiAudioIterator {
    /// Shared sequencer.
    sequencer: Arc<Mutex<MidiFileSequencer>>,
    /// Current position.
    position: Arc<Mutex<usize>>,
    /// Pause state.
    paused: Arc<Mutex<bool>>,
    /// Pre-rendered sample buffer.
    buffer: Vec<f32>,
    /// Current offset in buffer.
    buffer_offset: usize,
    /// Sample rate.
    sample_rate: u32,
}

impl Iterator for MidiAudioIterator {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // Refill buffer if needed
        if self.buffer_offset >= self.buffer.len() {
            self.buffer_offset = 0;

            // Check if paused
            let is_paused = self.paused.lock().map(|p| *p).unwrap_or(false);

            if is_paused {
                // Output silence when paused
                self.buffer.clear();
                self.buffer.resize(8192, 0.0); // 4096 stereo samples
            } else if let Ok(mut seq) = self.sequencer.lock() {
                // Render next chunk of samples
                let chunk_size = 4096;
                let mut left = vec![0.0; chunk_size];
                let mut right = vec![0.0; chunk_size];

                seq.render(&mut left, &mut right);

                // Interleave stereo samples
                self.buffer.clear();
                self.buffer.reserve(chunk_size * 2);
                for i in 0..chunk_size {
                    self.buffer.push(left[i]);
                    self.buffer.push(right[i]);
                }

                // Update position
                if let Ok(mut pos) = self.position.lock() {
                    *pos += chunk_size;
                }
            } else {
                // Failed to lock - output silence
                self.buffer.clear();
                self.buffer.resize(8192, 0.0);
            }
        }

        // Return next sample from buffer
        let sample = self.buffer.get(self.buffer_offset).copied();
        self.buffer_offset += 1;
        sample
    }
}

impl Source for MidiAudioIterator {
    fn current_frame_len(&self) -> Option<usize> {
        // Infinite stream
        None
    }

    fn channels(&self) -> u16 {
        2 // Stereo
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        // Unknown duration (depends on MIDI file)
        None
    }
}

impl Decodable for MidiAudioSource {
    type DecoderItem = f32;
    type Decoder = MidiAudioIterator;

    fn decoder(&self) -> Self::Decoder {
        MidiAudioIterator {
            sequencer: self.sequencer.clone(),
            position: self.position.clone(),
            paused: self.paused.clone(),
            buffer: Vec::new(),
            buffer_offset: 0,
            sample_rate: self.sample_rate,
        }
    }
}

/// MIDI playback state.
#[derive(Resource, Default)]
pub struct MidiState {
    /// Current MIDI audio source entity (if any).
    audio_entity: Option<Entity>,
    /// Current audio source handle (for pause/resume control).
    audio_source_handle: Option<Handle<MidiAudioSource>>,
    /// Cached SoundFont (loaded once for performance).
    soundfont: Option<Arc<SoundFont>>,
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
    mut commands: Commands,
    mut state: ResMut<MidiState>,
    soundfont_path: Res<SoundfontPath>,
    mut audio_assets: ResMut<Assets<MidiAudioSource>>,
) {
    match event.event() {
        MidiPlaybackEvent::Play { midi_data, looping } => {
            // Stop any existing playback
            if let Some(entity) = state.audio_entity {
                commands.entity(entity).despawn();
                state.audio_entity = None;
            }

            // Load SoundFont if not cached
            if state.soundfont.is_none() {
                match load_soundfont(&soundfont_path.0) {
                    Ok(sf) => state.soundfont = Some(sf),
                    Err(e) => {
                        error!("Failed to load SoundFont: {}", e);
                        return;
                    }
                }
            }

            // Create MIDI audio source
            let soundfont = state.soundfont.as_ref().unwrap().clone();
            match MidiAudioSource::new(soundfont, midi_data, *looping) {
                Ok(source) => {
                    // Add source to asset storage
                    let handle = audio_assets.add(source);

                    // Spawn audio player entity
                    let entity = commands
                        .spawn((AudioPlayer(handle.clone()), PlaybackSettings::LOOP))
                        .id();

                    state.audio_entity = Some(entity);
                    state.audio_source_handle = Some(handle);
                    info!("Started MIDI playback (looping: {})", looping);
                }
                Err(e) => {
                    error!("Failed to create MIDI audio source: {}", e);
                }
            }
        }
        MidiPlaybackEvent::Stop => {
            if let Some(entity) = state.audio_entity {
                commands.entity(entity).despawn();
                state.audio_entity = None;
                state.audio_source_handle = None;
                info!("MIDI playback stopped");
            } else {
                info!("No MIDI playback to stop");
            }
        }
        MidiPlaybackEvent::Pause => {
            if let Some(ref handle) = state.audio_source_handle {
                if let Some(source) = audio_assets.get(handle) {
                    if !source.is_paused() {
                        source.set_paused(true);
                        info!("MIDI playback paused");
                    } else {
                        info!("MIDI playback already paused");
                    }
                }
            } else {
                info!("No MIDI playback to pause");
            }
        }
        MidiPlaybackEvent::Resume => {
            if let Some(ref handle) = state.audio_source_handle {
                if let Some(source) = audio_assets.get(handle) {
                    if source.is_paused() {
                        source.set_paused(false);
                        info!("MIDI playback resumed");
                    } else {
                        info!("MIDI playback not paused");
                    }
                }
            } else {
                info!("No MIDI playback to resume");
            }
        }
    }
}

/// Load a SoundFont from disk.
fn load_soundfont(path: &str) -> Result<Arc<SoundFont>, String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("Failed to open SoundFont file: {}", e))?;
    let soundfont =
        SoundFont::new(&mut file).map_err(|e| format!("Failed to load SoundFont: {}", e))?;
    Ok(Arc::new(soundfont))
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
        let plugin = MidiPlugin::new("test.sf2");
        assert_eq!(plugin.soundfont_path, "test.sf2");
    }

    #[test]
    fn test_midi_state_default() {
        let state = MidiState::default();
        assert!(state.audio_entity.is_none());
        assert!(state.audio_source_handle.is_none());
        assert!(state.soundfont.is_none());
    }

    #[test]
    fn test_midi_playback_event_variants() {
        // Just test that we can construct all event variants
        let _play = MidiPlaybackEvent::Play {
            midi_data: vec![0x4d, 0x54, 0x68, 0x64], // "MThd" MIDI header
            looping: true,
        };
        let _stop = MidiPlaybackEvent::Stop;
        let _pause = MidiPlaybackEvent::Pause;
        let _resume = MidiPlaybackEvent::Resume;
    }
}
