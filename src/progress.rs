//! What the player has done, per instrument (`guitar-6`, `bass-4`...), saved locally as
//! `progress.json`: the drills with their best accuracy and speed, and for every note the
//! answers, the time to answer and the hints.
use std::collections::{BTreeMap, BTreeSet};

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
    /// The tempo of the metronome for the next round, 0 before the first round in time; see
    /// [`crate::tempo`].
    pub bpm: u32,
    /// The tempo to reach, set by the player; 0 is the default.
    pub target_bpm: u32,
    /// The rounds under the goal in a row at `bpm`.
    pub misses: u8,
    /// A clean round at the target tempo.
    pub target_reached: bool,
    /// The fastest clean round.
    pub best_bpm: u32,
    /// The notes per minute of the fastest clean round (the tempo times the notes per beat), for
    /// the tier of [`crate::tempo::Tier`].
    pub best_clean_npm: u32,
}

impl DrillRecord {
    /// 0 to 3 stars, by the best accuracy, see [`crate::exercises::STAR_ACCURACY`].
    pub fn stars(&self) -> u8 {
        if self.plays == 0 { 0 } else { crate::exercises::stars(self.best_accuracy) }
    }
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
    /// The finger drills of version 0.2 by number, moved to `exercises` on load.
    #[serde(skip_serializing)]
    pub drills: BTreeMap<usize, DrillRecord>,
    /// By the key of the exercise, see [`crate::exercises`].
    pub exercises: BTreeMap<String, DrillRecord>,
    /// The exercises opened at random.
    pub unlocked: BTreeSet<String>,
    /// The notes cleared in the learning order of [`crate::music::learning_order`].
    pub learned: usize,
    /// "Learn notes": reading a note and playing it, by the full name of the sounding note.
    pub reading: BTreeMap<String, NoteStats>,
    /// "Find them all": every place of a note on the fretboard.
    pub finding: BTreeMap<String, NoteStats>,
}

impl InstrumentProgress {
    /// A star at least: the next exercise opens.
    pub fn done(&self, key: &str) -> bool {
        self.exercises.get(key).is_some_and(|r| r.stars() > 0)
    }

    pub fn total_stars(&self) -> u32 {
        self.exercises.values().map(|r| r.stars() as u32).sum()
    }

    /// The numbered drills of version 0.2 under their keys; the orders that are rotations of
    /// each other are one drill now, the best of them is kept.
    fn migrate(&mut self) {
        for (id, record) in std::mem::take(&mut self.drills) {
            let Some(key) = crate::exercises::legacy_finger_key(id) else { continue };
            let kept = self.exercises.entry(key).or_default();
            if (record.best_accuracy, record.plays) > (kept.best_accuracy, kept.plays) {
                let plays = kept.plays + record.plays;
                *kept = DrillRecord { plays, ..record };
            } else {
                kept.plays += record.plays;
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Resource)]
#[serde(default)]
pub struct Progress {
    pub instruments: BTreeMap<String, InstrumentProgress>,
    /// The practice of each day since the UNIX epoch, every instrument together, see
    /// [`crate::practice`].
    pub days: BTreeMap<u64, crate::practice::Day>,
    /// The rounds of the drills, the oldest first.
    pub sessions: Vec<crate::practice::Session>,
}

impl Progress {
    pub fn of(&mut self, instrument_key: &str) -> &mut InstrumentProgress {
        self.instruments.entry(instrument_key.to_string()).or_default()
    }

    pub fn get(&self, instrument_key: &str) -> Option<&InstrumentProgress> {
        self.instruments.get(instrument_key)
    }

    pub fn load() -> Self {
        let mut progress: Progress = crate::storage::load("progress").unwrap_or_default();
        progress.instruments.values_mut().for_each(InstrumentProgress::migrate);
        progress
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
        p.of("guitar-6").exercises.insert("fingers/0/1-2-3-4".into(), DrillRecord { plays: 1, best_accuracy: 0.9, ..default() });
        let json = serde_json::to_string(&p).unwrap();
        let back: Progress = serde_json::from_str(&json).unwrap();
        assert_eq!(back, p);
        assert!(back.get("guitar-6").unwrap().done("fingers/0/1-2-3-4"));
        assert_eq!(back.get("guitar-6").unwrap().total_stars(), 2);
    }

    #[test]
    fn migration() {
        // Version 0.2: 1-2-3-4 (0) and its rotation 2-3-4-1, and 4-3-2-1 (1).
        let json = r#"{"instruments":{"guitar-6":{"drills":{
            "0":{"plays":2,"done":true,"best_accuracy":0.95},
            "1":{"plays":1,"best_accuracy":0.5}
        }}}}"#;
        let mut p: Progress = serde_json::from_str(json).unwrap();
        p.instruments.values_mut().for_each(InstrumentProgress::migrate);
        let g = p.get("guitar-6").unwrap();
        assert!(g.drills.is_empty());
        assert_eq!(g.exercises["fingers/0/1-2-3-4"].plays, 2);
        assert_eq!(g.exercises["fingers/0/4-3-2-1"].best_accuracy, 0.5);
        assert!(!serde_json::to_string(&p).unwrap().contains("\"drills\""));
    }
}
