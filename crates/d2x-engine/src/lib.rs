//! # D2X Engine
//!
//! Core game engine for D2X-RS using Bevy ECS.
//!
//! This crate implements all game systems as Bevy plugins.

pub mod level;
pub mod physics;
pub mod objects;
pub mod weapons;
pub mod ai;
pub mod collision;
pub mod rendering;
pub mod audio;

use bevy::prelude::*;

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
