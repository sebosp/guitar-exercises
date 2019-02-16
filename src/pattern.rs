
/// `GuitarExercisePattern` provides different patterns to practice a scale
#[derive(Clone, Debug)]
pub struct GuitarExercisePattern {
    name: String,
    difficulty: u8,
}

impl Default for GuitarExercisePattern {
    fn default() -> GuitarExercisePattern {
        GuitarExercisePattern{
            name: "Sequential".to_string(),
            difficulty: 0u8,
        }
    }
}

impl GuitarExercisePattern {
    pub fn random() -> GuitarExercisePattern {
        unimplemented!();
    }
}