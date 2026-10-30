//! What the player has done, per instrument (`guitar-6`, `bass-4`...), saved locally as
//! `progress.json`: the drills with their best accuracy and speed, and for every note the
//! answers, the time to answer and the hints.
use std::collections::BTreeMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::storage::now_unix_seconds;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DrillRecord {
    pub plays: u32,
    /// Reached the goal once: the next level opens.
    pub done: bool,
    /// 0 to 1.
    pub best_accuracy: f32,
    pub best_notes_per_minute: f32,
    pub last_accuracy: f32,
    pub last_notes_per_minute: f32,
    pub last_played: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct NoteStats {
    pub correct: u32,
    pub wrong: u32,
    /// The seconds to the right answers, summed.
    pub total_time: f32,
    pub hints: u32,
    pub last_played: u64,
}

impl NoteStats {
    pub fn answers(&self) -> u32 {
        self.correct + self.wrong
    }

    pub fn accuracy(&self) -> Option<f32> {
        (self.answers() > 0).then(|| self.correct as f32 / self.answers() as f32)
    }

    pub fn average_time(&self) -> Option<f32> {
        (self.correct > 0).then(|| self.total_time / self.correct as f32)
    }

    pub fn record(&mut self, correct: bool, seconds: f32, hinted: bool) {
        if correct {
            self.correct += 1;
            self.total_time += seconds;
        } else {
            self.wrong += 1;
        }
        if hinted {
            self.hints += 1;
        }
        self.last_played = now_unix_seconds();
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InstrumentProgress {
    /// By level id, see [`crate::drills::levels`].
    pub drills: BTreeMap<usize, DrillRecord>,
    /// The notes cleared in the learning order of [`crate::music::learning_order`].
    pub learned: usize,
    /// "Learn notes": reading a note and playing it, by the full name of the sounding note.
    pub reading: BTreeMap<String, NoteStats>,
    /// "Find them all": every place of a note on the fretboard.
    pub finding: BTreeMap<String, NoteStats>,
}

impl InstrumentProgress {
    pub fn drill_done(&self, id: usize) -> bool {
        self.drills.get(&id).is_some_and(|r| r.done)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Resource)]
#[serde(default)]
pub struct Progress {
    pub instruments: BTreeMap<String, InstrumentProgress>,
}

impl Progress {
    pub fn of(&mut self, instrument_key: &str) -> &mut InstrumentProgress {
        self.instruments.entry(instrument_key.to_string()).or_default()
    }

    pub fn get(&self, instrument_key: &str) -> Option<&InstrumentProgress> {
        self.instruments.get(instrument_key)
    }

    pub fn load() -> Self {
        crate::storage::load("progress").unwrap_or_default()
    }

    pub fn save(&self) {
        crate::storage::save("progress", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn note_stats() {
        let mut s = NoteStats::default();
        assert_eq!(s.accuracy(), None);
        s.record(true, 2.0, false);
        s.record(false, 9.0, true);
        s.record(true, 4.0, false);
        assert_eq!(s.accuracy(), Some(2.0 / 3.0));
        assert_eq!(s.average_time(), Some(3.0));
        assert_eq!(s.hints, 1);
    }

    #[test]
    fn round_trip() {
        let mut p = Progress::default();
        p.of("guitar-6").reading.entry("F#3".into()).or_default().record(true, 1.0, false);
        p.of("guitar-6").drills.insert(3, DrillRecord { done: true, ..default() });
        let json = serde_json::to_string(&p).unwrap();
        let back: Progress = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
        assert!(back.get("guitar-6").unwrap().drill_done(3));
    }
}
