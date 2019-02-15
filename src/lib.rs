
extern crate pitch_calc;
extern crate chords;
extern crate rand;
extern crate num_traits;

pub mod fretboard;
pub mod hand;
pub mod scale;

use fretboard::*;
use hand::*;
use scale::*;

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
