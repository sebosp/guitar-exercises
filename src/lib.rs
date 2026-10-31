//! Guitar Exercises: finger drills and note reading for the guitar and the bass, checked by
//! listening to the instrument.
// The queries of Bevy systems.
#![allow(clippy::type_complexity)]
use bevy::prelude::*;

pub mod audio;
pub mod diagram;
pub mod drills;
pub mod exercises;
pub mod fretboard;
pub mod learn;
pub mod metronome;
pub mod music;
pub mod pitch;
pub mod progress;
pub mod scales;
pub mod screens;
pub mod settings;
pub mod staff;
pub mod storage;
pub mod tempo;
pub mod theme;
pub mod tuner;
pub mod ui;
pub mod zoom;

pub struct GuitarExercisesPlugin;

impl Plugin for GuitarExercisesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(settings::Settings::load()).insert_resource(progress::Progress::load()).add_plugins((
            ui::UiPlugin,
            audio::AudioInputPlugin,
            metronome::MetronomePlugin,
            tuner::TunerPlugin,
            zoom::ZoomPlugin,
            fretboard::FretboardPlugin,
            screens::ScreensPlugin,
        ));
    }
}
