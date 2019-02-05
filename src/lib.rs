
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_gets_strings() {
    }
}