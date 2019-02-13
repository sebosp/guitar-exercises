use std::fmt;

use chords::scale::*;
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
use num_traits::cast::ToPrimitive;

/// `ScaleCategory` for Major, Minor, etc on a given note/pitch
#[derive(Debug, Clone)]
pub struct ScaleCategory {
    pub category: String,
    pub scale_notes: Vec<Letter>,
}

impl Default for ScaleCategory {
    fn default() -> ScaleCategory {
        ScaleCategory{
            scale_notes: vec![],
            category: "Major".to_string()
        }
    }
}

/// `ScaleCategory` shows a 
impl ScaleCategory {
    pub fn random() -> ScaleCategory {
        let mut rng = rand::thread_rng();
        let scale_rng: usize = rng.gen();
        let scales = chords::supported_scales();
        let total_scales = scales.len();
        let random_scale = &scales[scale_rng % total_scales];
        let starting_note = Letter::from_i64(rng.gen_range(0, 12) as i64).unwrap();
        println!("Working on scale for Letter: {:?}", starting_note);
        let scale_notes:Vec<Letter> = chords::scale::get_scale(random_scale).iter().map(|offset|
            Letter::from_i64(starting_note.to_i64().unwrap() + *offset as i64).unwrap()
        ).collect();
        ScaleCategory{
            scale_notes: scale_notes,
            category: random_scale.clone(),
        }
    }
}


impl fmt::Display for ScaleCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {:?})", self.category, self.scale_notes)
    }
}