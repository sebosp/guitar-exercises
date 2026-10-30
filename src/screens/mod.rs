//! The screens of the game, one module each.
use bevy::prelude::*;

pub mod drill;
pub mod drills;
pub mod find;
pub mod learn;
pub mod menu;
pub mod settings;
pub mod stats;

/// The note the current game waits for, for the tests that play by themselves.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Expected {
    pub midi: Option<i32>,
    pub position: Option<crate::music::Position>,
}

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Expected>().add_plugins((
            menu::MenuPlugin,
            settings::SettingsScreenPlugin,
            drills::DrillsPlugin,
            drill::DrillPlugin,
            learn::LearnPlugin,
            find::FindPlugin,
            stats::StatsPlugin,
        ));
    }
}
