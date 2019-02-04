use pitch_calc::{
    Hz,
    Letter,
    LetterOctave,
    ScaledPerc,
    Step,
    Octave,
};
use chords::*;

/// `ScaleCategory` for Major, Minor, etc on a given note/pitch
pub struct ScaleCategory {
    note: Letter,
    category: String
}

impl Default for ScaleCategory {
    fn default() -> ScaleCategory {
        ScaleCategory{
            note: Letter::C,
            category: "Major".to_string()
        }
    }
}
