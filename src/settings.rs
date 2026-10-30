//! The settings of the player, saved locally as `settings.json`.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::music::{Instrument, InstrumentKind, TuningPreset};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource)]
#[serde(default)]
pub struct Settings {
    pub instrument: InstrumentKind,
    pub strings: usize,
    pub tuning: TuningPreset,
    /// The frets shown and used, 12 by default.
    pub frets: u8,
    /// The fretboard mirrored: the nut on the right.
    pub left_handed: bool,
    /// Listen to the instrument; without it the notes are answered by tapping the fretboard.
    pub microphone: bool,
    /// The level under which the microphone hears silence, from 1 (sensitive) to 10.
    pub mic_gate: u8,
    /// Seconds without an answer before a hint.
    pub hint_delay: f32,
    /// The times a drill goes up and down the strings, one fret higher each time.
    pub drill_shifts: u8,
    /// The note names on the fretboard in the drills.
    pub show_note_names: bool,
    /// Sharps and flats in the note games, after the natural notes.
    pub accidentals: bool,
    /// The right answers in a row that clear a new note.
    pub clear_streak: u8,
}

impl Default for Settings {
    fn default() -> Self {
        Settings::for_instrument(InstrumentKind::Guitar)
    }
}

impl Settings {
    pub fn for_instrument(kind: InstrumentKind) -> Self {
        Settings {
            instrument: kind,
            strings: kind.default_strings(),
            tuning: TuningPreset::Standard,
            frets: 12,
            left_handed: false,
            microphone: true,
            mic_gate: 3,
            hint_delay: 5.0,
            drill_shifts: 1,
            show_note_names: false,
            accidentals: false,
            clear_streak: 3,
        }
    }

    pub fn instrument(&self) -> Instrument {
        Instrument::new(self.instrument, self.strings, self.tuning, self.frets)
    }

    /// The RMS level of the microphone gate.
    pub fn gate_level(&self) -> f32 {
        0.002 * self.mic_gate as f32
    }

    /// Back in the ranges after a load of an older or edited file.
    pub fn sanitize(&mut self) {
        let counts = self.instrument.string_counts();
        if !counts.contains(&self.strings) {
            self.strings = self.instrument.default_strings();
        }
        self.frets = self.frets.clamp(12, 24);
        self.mic_gate = self.mic_gate.clamp(1, 10);
        self.hint_delay = self.hint_delay.clamp(1.0, 30.0);
        self.drill_shifts = self.drill_shifts.clamp(1, 6);
        self.clear_streak = self.clear_streak.clamp(1, 10);
    }

    pub fn load() -> Self {
        let mut settings: Settings = crate::storage::load("settings").unwrap_or_default();
        settings.sanitize();
        settings
    }

    pub fn save(&self) {
        crate::storage::save("settings", self);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_and_sanitize() {
        let mut s = Settings::for_instrument(InstrumentKind::Bass);
        s.strings = 5;
        let json = serde_json::to_string(&s).unwrap();
        let back: Settings = serde_json::from_str(&json).unwrap();
        assert_eq!(back, s);
        // Missing fields take the defaults.
        let partial: Settings = serde_json::from_str(r#"{"instrument":"Bass","strings":9}"#).unwrap();
        let mut partial = partial;
        partial.sanitize();
        assert_eq!(partial.strings, 4);
        assert_eq!(partial.frets, 12);
    }
}
