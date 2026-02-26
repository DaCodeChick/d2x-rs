//! First-time setup system for converting Descent game assets.
//!
//! This module handles the initial asset conversion process that runs when the player
//! first launches the game. It extracts assets from the original Descent installation
//! and converts them to modern formats for faster loading.
//!
//! # Conversion Process
//!
//! 1. **Detection**: Check if converted assets exist
//! 2. **Prompt**: Ask user to select Descent installation folder
//! 3. **Extract**: Read HOG/DHF archives
//! 4. **Convert**:
//!    - HMP → MIDI (Descent music format to standard MIDI)
//!    - PIG → PNG (indexed textures to true color)
//!    - POF → GLB (Descent models to glTF binary)
//!    - PCM → WAV (8-bit sound effects to 16-bit)
//! 5. **Organize**: Save to clean asset directory structure
//!
//! # Directory Structure
//!
//! After conversion, assets are organized as:
//!
//! ```text
//! assets/
//! ├── music/          # MIDI files
//! ├── textures/       # PNG files
//! ├── models/         # GLB files
//! ├── sounds/         # WAV files
//! └── levels/         # Original level files (RDL/RL2)
//! ```

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use std::path::PathBuf;
use tracing::{info, warn};

/// Plugin for handling first-time setup and asset conversion.
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<AssetStatus>()
            .init_resource::<SetupMode>()
            .init_resource::<ConversionProgress>()
            .add_systems(Startup, check_assets.after(crate::initialize))
            .add_systems(Update, setup_ui)
            .add_systems(Update, handle_setup);
    }
}

/// Status of converted assets.
#[derive(Resource, Default)]
pub struct AssetStatus {
    /// Path to converted assets directory.
    pub assets_path: PathBuf,
    /// Path to original Descent installation (if selected).
    pub source_path: Option<PathBuf>,
    /// Whether all required assets are present.
    pub ready: bool,
    /// Whether we've checked for assets yet.
    pub checked: bool,
}

/// Current setup mode.
#[derive(Resource, Default, PartialEq, Eq)]
pub enum SetupMode {
    #[default]
    /// Not yet checked.
    NotChecked,
    /// Assets found, no setup needed.
    Complete,
    /// Needs setup, showing prompt.
    NeedsSetup,
    /// Currently converting.
    Converting,
}

/// Conversion progress tracking.
#[derive(Resource, Default)]
pub struct ConversionProgress {
    /// Current file being converted.
    pub current_file: String,
    /// Number of files converted so far.
    pub files_done: usize,
    /// Total number of files to convert.
    pub files_total: usize,
    /// Current conversion stage.
    pub stage: ConversionStage,
}

/// Stages of asset conversion.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub enum ConversionStage {
    #[default]
    Idle,
    ExtractingArchives,
    ConvertingMusic,
    ConvertingTextures,
    ConvertingModels,
    ConvertingSounds,
    CopyingLevels,
    Finished,
}

/// Check if converted assets exist.
fn check_assets(mut status: ResMut<AssetStatus>, mut mode: ResMut<SetupMode>) {
    if status.checked {
        return;
    }

    info!("Checking for converted assets at: {:?}", status.assets_path);

    // Check for required directories
    let assets_path = &status.assets_path;
    let required_dirs = ["music", "textures", "models", "sounds", "levels"];

    let all_exist = required_dirs.iter().all(|dir| {
        let path = assets_path.join(dir);
        path.exists() && path.is_dir()
    });

    status.checked = true;

    if all_exist {
        info!("Converted assets found, ready to play");
        status.ready = true;
        *mode = SetupMode::Complete;
    } else {
        info!("Converted assets not found, setup required");
        *mode = SetupMode::NeedsSetup;
    }
}

/// Handle setup flow.
fn handle_setup(
    mode: Res<SetupMode>,
    mut status: ResMut<AssetStatus>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    if *mode == SetupMode::NeedsSetup {
        // Placeholder: Press S to skip setup (for testing)
        if keyboard.just_pressed(KeyCode::KeyS) {
            warn!("Setup skipped by user (development mode)");
            status.ready = true;
        }
    }
}

/// Display setup UI with egui.
fn setup_ui(
    mut contexts: EguiContexts,
    mode: Res<SetupMode>,
    mut status: ResMut<AssetStatus>,
    progress: Res<ConversionProgress>,
) {
    // Only show UI during setup stages
    if *mode != SetupMode::NeedsSetup && *mode != SetupMode::Converting {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);

            ui.heading("D2X-RS - First-Time Setup");
            ui.add_space(20.0);

            if *mode == SetupMode::NeedsSetup {
                ui.label("Welcome to D2X-RS!");
                ui.add_space(10.0);
                ui.label(
                    "To play, we need to convert assets from the original Descent installation.",
                );
                ui.add_space(10.0);
                ui.label("This is a one-time process (like OpenMW or OpenRCT2).");
                ui.add_space(30.0);

                if let Some(ref source) = status.source_path {
                    ui.label(format!("Selected: {}", source.display()));
                    ui.add_space(10.0);

                    if ui.button("Start Conversion").clicked() {
                        info!("Starting asset conversion from: {:?}", source);
                        // TODO: Trigger conversion
                    }

                    ui.add_space(10.0);

                    if ui.button("Choose Different Folder").clicked() {
                        // Open file dialog
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select Descent Installation Folder")
                            .pick_folder()
                        {
                            info!("User selected Descent folder: {:?}", path);
                            status.source_path = Some(path);
                        }
                    }
                } else {
                    if ui.button("Select Descent Installation Folder").clicked() {
                        // Open file dialog
                        if let Some(path) = rfd::FileDialog::new()
                            .set_title("Select Descent Installation Folder")
                            .pick_folder()
                        {
                            info!("User selected Descent folder: {:?}", path);
                            status.source_path = Some(path);
                        }
                    }
                }

                ui.add_space(20.0);
                ui.label("Press 'S' to skip (development mode)");
            } else if *mode == SetupMode::Converting {
                ui.label(format!("Converting Assets... ({:?})", progress.stage));
                ui.add_space(20.0);

                let progress_value =
                    progress.files_done as f32 / progress.files_total.max(1) as f32;
                ui.add(egui::ProgressBar::new(progress_value).show_percentage());

                ui.add_space(10.0);
                ui.label(format!(
                    "{} / {} files",
                    progress.files_done, progress.files_total
                ));

                if !progress.current_file.is_empty() {
                    ui.add_space(10.0);
                    ui.label(format!("Current: {}", progress.current_file));
                }
            }
        });
    });
}
