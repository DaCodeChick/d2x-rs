mod setup;
mod ui;
mod video;

use bevy::prelude::*;
use d2x_engine::D2xEnginePlugin;
use setup::{AssetStatus, SetupMode, SetupPlugin};
use std::path::PathBuf;
use tracing::info;
use ui::MenuPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "D2X-RS - Descent Engine Rewrite".to_string(),
                resolution: (1280, 720).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(MenuPlugin)
        .add_plugins(SetupPlugin)
        .add_plugins(D2xEnginePlugin)
        .add_systems(Startup, initialize)
        .add_systems(Update, start_game)
        .run();
}

/// Initialize the client on startup.
fn initialize(mut commands: Commands) {
    info!("D2X-RS Client started");

    // Determine assets path (in user's data directory)
    let assets_path = get_assets_path();
    info!("Assets directory: {:?}", assets_path);

    // Initialize asset status
    commands.insert_resource(AssetStatus {
        assets_path,
        source_path: None,
        ready: false,
        checked: false,
    });
}

/// Start the actual game once setup is complete.
fn start_game(
    mut commands: Commands,
    status: Res<AssetStatus>,
    mode: Res<SetupMode>,
    mut ran_once: Local<bool>,
) {
    if *ran_once || !status.ready || *mode != SetupMode::Complete {
        return;
    }
    *ran_once = true;

    info!("Starting game with assets from: {:?}", status.assets_path);

    // Setup camera
    commands.spawn((Camera3d::default(), Name::new("MainCamera")));

    // Setup light
    commands.spawn((
        PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        Name::new("MainLight"),
    ));

    // TODO: Load main menu, start gameplay, etc.
}

/// Get the path where converted assets should be stored.
///
/// This uses platform-specific directories:
/// - Linux: `~/.local/share/d2x-rs/assets`
/// - macOS: `~/Library/Application Support/d2x-rs/assets`
/// - Windows: `%APPDATA%/d2x-rs/assets`
fn get_assets_path() -> PathBuf {
    #[cfg(target_os = "linux")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join(".local/share/d2x-rs/assets")
        } else {
            PathBuf::from("./assets")
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(home) = std::env::var_os("HOME") {
            PathBuf::from(home).join("Library/Application Support/d2x-rs/assets")
        } else {
            PathBuf::from("./assets")
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Some(appdata) = std::env::var_os("APPDATA") {
            PathBuf::from(appdata).join("d2x-rs/assets")
        } else {
            PathBuf::from("./assets")
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        PathBuf::from("./assets")
    }
}
