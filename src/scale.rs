use pitch_calc::{
    Hz,
    Letter,
    LetterOctave,
    ScaledPerc,
    Step,
    Octave,
};
use chords::*;
use rand::Rng;
use num_traits::cast::FromPrimitive;

/// `ScaleCategory` for Major, Minor, etc on a given note/pitch
#[derive(Debug)]
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

impl ScaleCategory {
    pub fn random() -> ScaleCategory {
        let mut rng = rand::thread_rng();
        let scale_nth: usize = rng.gen();
        let note_nth: i64 = rng.gen();
        let scales = chords::supported_scales();
        let total_scales = scales.len();
        ScaleCategory{
            note: Letter::from_i64(note_nth).unwrap(), // XXX: Can this actualy return None?
            category: scales[scale_nth % total_scales].clone(),
        }
    }
}