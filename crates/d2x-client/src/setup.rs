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
//! 3. **Extract**: Read HOG/DHF archives and MVL video archives
//! 4. **Convert**:
//!    - HMP → MIDI (Descent music format to standard MIDI)
//!    - PIG → TGA (indexed textures to true color)
//!    - POF → GLB (Descent models to glTF binary)
//!    - PCM → WAV (8-bit sound effects to 16-bit)
//!    - MVE → MP4 (Interplay videos to H.264 using FFmpeg)
//! 5. **Organize**: Save to clean asset directory structure
//!
//! # Video Conversion
//!
//! MVE files are converted to modern MP4/H.264 format for several reasons:
//! - **Performance**: Hardware-accelerated H.264 decode vs. software MVE decode
//! - **File size**: Modern compression is 10-50x smaller than 1990s MVE codec
//! - **Integration**: Works seamlessly with Bevy video playback plugins
//! - **Quality**: Opportunity to enhance/upscale during conversion
//!
//! The conversion uses the `ffmpeg-next` Rust crate with the `BUILD` feature enabled.
//! This **automatically compiles and statically links FFmpeg** at build time, so users
//! don't need to install FFmpeg separately. The FFmpeg libraries are bundled into the
//! game executable.
//!
//! **Note**: Video conversion is only available when the `cutscenes` feature is enabled.
//! This allows builds without cutscene support to avoid the large FFmpeg dependency.
//!
//! # Directory Structure
//!
//! After conversion, assets are organized as:
//!
//! ```text
//! assets/
//! ├── music/          # MIDI files (from HMP)
//! ├── textures/       # TGA files (from PIG)
//! ├── models/         # GLB files (from POF)
//! ├── sounds/         # WAV files (from PCM)
//! ├── videos/         # MP4 files (from MVE)
//! └── levels/         # Original level files (RDL/RL2)
//! ```

use bevy::prelude::*;
use std::path::PathBuf;
use tracing::{info, warn};

use crate::ui::{Menu, MenuItem, MenuState};

/// Plugin for handling first-time setup and asset conversion.
pub struct SetupPlugin;

impl Plugin for SetupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AssetStatus>()
            .init_resource::<SetupMode>()
            .init_resource::<ConversionProgress>()
            .add_systems(Startup, check_assets.after(crate::initialize))
            .add_systems(Update, show_setup_message)
            .add_systems(Update, show_setup_menu)
            .add_systems(Update, handle_setup_input);
    }
}

/// Status of converted assets.
#[derive(Resource, Default)]
pub struct AssetStatus {
    /// Path to converted assets directory.
    pub assets_path: PathBuf,
    /// Path to original Descent installation (if selected).
    #[allow(dead_code)] // Future UI feature
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
    #[allow(dead_code)] // Future UI feature
    Converting,
}

/// Conversion progress tracking.
#[derive(Resource, Default)]
#[allow(dead_code)] // Future UI feature
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
#[allow(dead_code)] // Future UI feature
pub enum ConversionStage {
    #[default]
    Idle,
    ExtractingArchives,
    ConvertingMusic,
    ConvertingTextures,
    ConvertingModels,
    ConvertingSounds,
    ConvertingVideos,
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
    let required_dirs = ["music", "textures", "models", "sounds", "videos", "levels"];

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

/// Show setup message once when setup is needed.
fn show_setup_message(mode: Res<SetupMode>, mut shown: Local<bool>) {
    if *mode == SetupMode::NeedsSetup && !*shown {
        println!("\n╔══════════════════════════════════════════════════════════╗");
        println!("║           D2X-RS - FIRST-TIME SETUP REQUIRED            ║");
        println!("╠══════════════════════════════════════════════════════════╣");
        println!("║                                                          ║");
        println!("║  To play, we need to convert assets from the original   ║");
        println!("║  Descent installation. This is a one-time process.      ║");
        println!("║                                                          ║");
        println!("║  TODO: Implement asset conversion pipeline              ║");
        println!("║        (Press 'S' to skip for now - dev mode)           ║");
        println!("║                                                          ║");
        println!("╚══════════════════════════════════════════════════════════╝\n");
        *shown = true;
    }
}

/// Handle setup flow input.
fn handle_setup_input(
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

/// Show the setup menu UI
fn show_setup_menu(
    mut commands: Commands,
    mode: Res<SetupMode>,
    mut menu_state: ResMut<MenuState>,
    mut shown: Local<bool>,
    existing_menu: Query<Entity, With<Menu>>,
) {
    // Only show menu when setup is needed
    if *mode != SetupMode::NeedsSetup {
        return;
    }

    // Create menu once
    if !*shown {
        // Despawn any existing menu
        for entity in existing_menu.iter() {
            commands.entity(entity).despawn();
        }

        // Create a test menu with various item types
        let menu = Menu::new("D2X-RS - First Time Setup")
            .with_subtitle("Please configure your game settings")
            .add_item(MenuItem::text("Welcome to D2X-RS!"))
            .add_item(MenuItem::text(""))
            .add_item(MenuItem::menu("Select Descent Installation Folder"))
            .add_item(MenuItem::menu("Start Conversion"))
            .add_item(MenuItem::text(""))
            .add_item(MenuItem::text("Options:"))
            .add_item(MenuItem::checkbox("Enable Enhanced Graphics", true))
            .add_item(MenuItem::checkbox("Enable Remastered Music", false))
            .add_item(MenuItem::slider("Volume", 75, 0, 100))
            .add_item(MenuItem::text(""))
            .add_item(MenuItem::menu("Skip (Dev Mode)"));

        let menu_entity = commands.spawn(menu).id();

        // Show the menu
        menu_state.active_menu = Some(menu_entity);
        menu_state.visible = true;
        menu_state.selected_index = 2; // Select first selectable item

        *shown = true;
        info!("Setup menu created and displayed");
    }
}
