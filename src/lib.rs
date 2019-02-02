
extern crate pitch_calc;
extern crate chords;

pub mod fretboard;
pub mod hand;

use fretboard::*;
use hand::*;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn it_gets_strings() {
    }
}