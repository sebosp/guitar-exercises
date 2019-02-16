
extern crate pitch_calc;
extern crate chords;
extern crate rand;
extern crate num_traits;
#[macro_use]
extern crate log;
extern crate clap;
extern crate stderrlog;
#[macro_use]
extern crate derive_builder;

pub mod fretboard;
pub mod hand;
pub mod scale;
pub mod pattern;

use fretboard::*;
use hand::*;
use scale::*;
use pattern::*;

/// `GuitarExerciseMode` can be a request for command line interface or http
#[derive(Clone, Debug)]
pub enum GuitarExerciseMode {
    Cli,
    Web
}

impl Default for GuitarExerciseMode {
    fn default() -> GuitarExerciseMode{
        GuitarExerciseMode::Cli
    }
}

/// `GuitarExerciseRequest` serves a request for an exercise
#[derive(Builder,Debug)]
#[builder(setter(prefix = "with"))]
pub struct GuitarExerciseRequest {
    mode: GuitarExerciseMode,
    scale: ScaleCategory,
    #[builder(default = "pitch_calc::Letter::C")]
    note: pitch_calc::Letter,
    pattern: GuitarExercisePattern,
}

impl Default for GuitarExerciseRequest {
    fn default() -> GuitarExerciseRequest {
        GuitarExerciseRequest{
            mode: GuitarExerciseMode::Cli,
            scale: ScaleCategory::default(),
            note: pitch_calc::Letter::C,
            pattern: GuitarExercisePattern::default(),
        }
    }
}

pub fn note_to_string(note: pitch_calc::Letter) -> String {
    match note {
        pitch_calc::Letter::C   => " C".to_string(),
        pitch_calc::Letter::Csh => "C#".to_string(),
        pitch_calc::Letter::Db  => "Db".to_string(),
        pitch_calc::Letter::D   => " D".to_string(),
        pitch_calc::Letter::Dsh => "D#".to_string(),
        pitch_calc::Letter::Eb  => "Eb".to_string(),
        pitch_calc::Letter::E   => " E".to_string(),
        pitch_calc::Letter::F   => " F".to_string(),
        pitch_calc::Letter::Fsh => "F#".to_string(),
        pitch_calc::Letter::Gb  => "Gb".to_string(),
        pitch_calc::Letter::G   => " G".to_string(),
        pitch_calc::Letter::Gsh => "G#".to_string(),
        pitch_calc::Letter::Ab  => "Ab".to_string(),
        pitch_calc::Letter::A   => " A".to_string(),
        pitch_calc::Letter::Ash => "A#".to_string(),
        pitch_calc::Letter::Bb  => "Bb".to_string(),
        pitch_calc::Letter::B   => " B".to_string(),
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_gets_strings() {
    }
}
