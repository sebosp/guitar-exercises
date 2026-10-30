//! Guitar Exercises on Android: full screen, the same plugin as the desktop and the web.
use bevy::prelude::*;
use bevy::window::WindowMode;
use guitar_exercises::GuitarExercisesPlugin;

#[bevy_main]
pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Guitar Exercises".into(),
                resizable: false,
                mode: WindowMode::BorderlessFullscreen(MonitorSelection::Primary),
                ..default()
            }),
            ..default()
        }))
        // The loop keeps running without input (not `WinitSettings::mobile()`): the microphone is
        // analyzed every frame.
        .add_plugins(GuitarExercisesPlugin)
        .run();
}
