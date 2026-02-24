use bevy::prelude::*;
use d2x_engine::D2xEnginePlugin;
use tracing::info;

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
        .add_plugins(D2xEnginePlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Setup camera
    commands.spawn(Camera3d::default());

    // Setup light
    commands.spawn(PointLight {
        intensity: 1500.0,
        shadows_enabled: true,
        ..default()
    });

    info!("D2X-RS Client started");
}
