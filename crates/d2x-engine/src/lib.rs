//! # D2X Engine
//!
//! Core game engine for D2X-RS using Bevy ECS.
//!
//! This crate implements all game systems as Bevy plugins.

pub mod ai;
pub mod audio;
pub mod collision;
pub mod level;
pub mod objects;
pub mod physics;
pub mod rendering;
pub mod weapons;

use bevy::prelude::*;
use tracing::info;

/// Main engine plugin
pub struct D2xEnginePlugin;

impl Plugin for D2xEnginePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_engine);
    }
}

fn setup_engine() {
    info!("D2X Engine initialized");
}
