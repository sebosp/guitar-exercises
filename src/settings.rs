//! The settings of the player, saved locally as `settings.json`.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::music::{Instrument, InstrumentKind, RETUNE_RANGE, TuningPreset};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Resource)]
#[serde(default)]
pub struct Settings {
    pub instrument: InstrumentKind,
    pub strings: usize,
    /// The semitones of each string from the standard tuning, the lowest string first: the
    /// presets set them all, the steppers of the settings one string.
    #[serde(deserialize_with = "offsets_or_preset")]
    pub tuning: Vec<i8>,
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
    /// The metronome in the drills, with the tempo of each drill going up to its target.
    pub metronome: bool,
    /// The drill notes per beat of the metronome: 1 quarter notes, 2 eighths, 4 sixteenths.
    pub notes_per_beat: u8,
    pub theme: crate::theme::Theme,
    /// The scale of the scale exercises.
    pub scale: crate::scales::Scale,
    /// The zoom over the scale that fits the window, see [`crate::zoom`].
    pub zoom: f32,
    /// The minutes of practice of a day for the streak.
    pub daily_goal_minutes: u32,
    /// The help was shown, on the first start.
    pub help_seen: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings::for_instrument(InstrumentKind::Guitar)
    }
}

/// The tuning as offsets, or as the name of a preset in the files of version 0.2.
fn offsets_or_preset<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Vec<i8>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Tuning {
        Offsets(Vec<i8>),
        Preset(TuningPreset),
    }
    Ok(match Tuning::deserialize(deserializer)? {
        Tuning::Offsets(offsets) => offsets,
        Tuning::Preset(preset) => preset.offsets(8),
    })
}

impl Settings {
    pub fn for_instrument(kind: InstrumentKind) -> Self {
        Settings {
            instrument: kind,
            strings: kind.default_strings(),
            tuning: vec![0; kind.default_strings()],
            frets: 12,
            left_handed: false,
            microphone: true,
            mic_gate: 3,
            hint_delay: 5.0,
            drill_shifts: 1,
            show_note_names: false,
            accidentals: false,
            clear_streak: 3,
            metronome: true,
            notes_per_beat: 1,
            theme: crate::theme::Theme::Dark,
            scale: crate::scales::Scale::default(),
            zoom: 1.0,
            daily_goal_minutes: 10,
            help_seen: false,
        }
    }

    pub fn instrument(&self) -> Instrument {
        Instrument::tuned(self.instrument, self.strings, &self.tuning, self.frets)
    }

    /// Every exercise of the drills page for this instrument and scale.
    pub fn catalog(&self) -> crate::exercises::Catalog {
        crate::exercises::catalog(&self.instrument(), self.scale)
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
        // A preset of an older file is read for 8 strings.
        if self.tuning.len() > self.strings {
            self.tuning.truncate(self.strings);
        }
        if self.tuning.len() != self.strings {
            self.tuning = vec![0; self.strings];
        }
        for offset in &mut self.tuning {
            *offset = (*offset).clamp(*RETUNE_RANGE.start(), *RETUNE_RANGE.end());
        }
        self.frets = self.frets.clamp(12, 24);
        self.mic_gate = self.mic_gate.clamp(1, 10);
        self.hint_delay = self.hint_delay.clamp(1.0, 30.0);
        self.drill_shifts = self.drill_shifts.clamp(1, 6);
        self.clear_streak = self.clear_streak.clamp(1, 10);
        self.notes_per_beat = self.notes_per_beat.clamp(1, 4);
        self.scale = self.scale.sanitized();
        if !self.zoom.is_finite() {
            self.zoom = 1.0;
        }
        self.daily_goal_minutes = self.daily_goal_minutes.clamp(1, 120);
        self.zoom = self.zoom.clamp(crate::zoom::MIN_ZOOM / 3.0, crate::zoom::MAX_ZOOM / 0.75);
    }

    pub fn load() -> Self {
        let mut settings: Settings = crate::storage::load("settings").unwrap_or_default();
        settings.sanitize();
        crate::theme::set(settings.theme);
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
        assert_eq!(partial.tuning, [0; 4]);
    }

    #[test]
    fn tunings() {
        // The files of version 0.2 have a preset.
        let mut old: Settings = serde_json::from_str(r#"{"instrument":"Bass","strings":5,"tuning":"Drop"}"#).unwrap();
        old.sanitize();
        assert_eq!(old.tuning, [-2, 0, 0, 0, 0]);
        assert_eq!(old.instrument().open[0], 21);
        let mut old: Settings = serde_json::from_str(r#"{"strings":6,"tuning":"HalfStepDown"}"#).unwrap();
        old.sanitize();
        assert_eq!(old.tuning, [-1; 6]);
        let mut wild = Settings { tuning: vec![40, -40, 0, 0], ..Settings::for_instrument(InstrumentKind::Bass) };
        wild.sanitize();
        assert_eq!(wild.tuning, [12, -12, 0, 0]);
    }
}
