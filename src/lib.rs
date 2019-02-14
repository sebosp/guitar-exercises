
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
        _ => "unimplemented".to_string()
    }
}
#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_gets_strings() {
    }
}