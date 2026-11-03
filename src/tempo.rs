//! The tempo of a drill with the metronome: the current tempo goes up 5 BPM after each clean
//! round until the target, stays after a round under the goal and goes down 5 BPM after a second
//! one in a row. Without Bevy.
//!
//! A round is clean when the notes are right (see [`crate::screens::drill::GOAL`]) and in time:
//! each note on its slot of the beat. The microphone hears a note some tens of milliseconds after
//! the pluck, so the slots are measured from the player's own average offset, not from the click.
use crate::progress::DrillRecord;

pub const START_BPM: u32 = 60;
pub const DEFAULT_TARGET_BPM: u32 = 100;
pub const MIN_BPM: u32 = 30;
pub const MAX_BPM: u32 = 240;
pub const BPM_STEP: u32 = 5;
/// The notes in time that make a round clean.
pub const TIMING_GOAL: f32 = 0.8;
/// The misses in a row at a tempo before it goes down.
pub const MISSES_TO_SLOW_DOWN: u8 = 2;

/// The notes per minute of a clean round for the tiers after Basic: Medium, Advanced, Legendary
/// (80 BPM in eighths, 120 BPM in eighths, 90 BPM in sixteenths).
pub const TIER_NOTES_PER_MINUTE: [u32; 3] = [160, 240, 360];

/// The tier of an exercise by its fastest clean round with the metronome, shown as rank pips on
/// its tile, as in the games.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    Basic,
    Medium,
    Advanced,
    Legendary,
}

impl Tier {
    pub const ALL: [Tier; 4] = [Tier::Basic, Tier::Medium, Tier::Advanced, Tier::Legendary];

    pub fn of(record: &DrillRecord) -> Option<Tier> {
        let npm = record.best_clean_npm;
        (npm > 0).then(|| match TIER_NOTES_PER_MINUTE.iter().filter(|&&t| npm >= t).count() {
            0 => Tier::Basic,
            1 => Tier::Medium,
            2 => Tier::Advanced,
            _ => Tier::Legendary,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Tier::Basic => "Basic",
            Tier::Medium => "Medium",
            Tier::Advanced => "Advanced",
            Tier::Legendary => "Legendary",
        }
    }

    /// The notes per minute it needs, in clean rounds.
    pub fn threshold(self) -> u32 {
        match self {
            Tier::Basic => 1,
            Tier::Medium => TIER_NOTES_PER_MINUTE[0],
            Tier::Advanced => TIER_NOTES_PER_MINUTE[1],
            Tier::Legendary => TIER_NOTES_PER_MINUTE[2],
        }
    }
}

/// What a round changed in the tempo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TempoChange {
    /// Clean: the next round this much faster.
    Faster(u32),
    /// Clean at the target tempo: the lesson is complete.
    TargetReached(u32),
    /// Under the goal once: the same tempo again.
    Again(u32),
    /// Under the goal twice in a row: slower.
    Slower(u32),
}

/// The tempo of a drill and its target, with the defaults for a drill never played in time.
pub fn tempo_of(record: &DrillRecord) -> (u32, u32) {
    let target = if record.target_bpm == 0 { DEFAULT_TARGET_BPM } else { record.target_bpm };
    let bpm = if record.bpm == 0 { START_BPM.min(target) } else { record.bpm };
    (bpm, target)
}

/// Sets the tempo by hand: the misses start again, the target is never under the tempo.
pub fn set_tempo(record: &mut DrillRecord, bpm: u32, target: u32) {
    let target = target.clamp(MIN_BPM, MAX_BPM);
    let bpm = bpm.clamp(MIN_BPM, MAX_BPM).min(target);
    if (bpm, target) != tempo_of(record) {
        record.misses = 0;
    }
    record.bpm = bpm;
    record.target_bpm = target;
}

/// The tempo after a round played at the current tempo.
pub fn after_round(record: &mut DrillRecord, clean: bool) -> TempoChange {
    let (bpm, target) = tempo_of(record);
    record.target_bpm = target;
    if clean {
        record.misses = 0;
        record.best_bpm = record.best_bpm.max(bpm);
        if bpm >= target {
            record.bpm = bpm;
            record.target_reached = true;
            return TempoChange::TargetReached(bpm);
        }
        record.bpm = (bpm + BPM_STEP).min(target);
        return TempoChange::Faster(record.bpm);
    }
    record.misses += 1;
    if record.misses >= MISSES_TO_SLOW_DOWN {
        record.misses = 0;
        record.bpm = bpm.saturating_sub(BPM_STEP).max(MIN_BPM);
        return TempoChange::Slower(record.bpm);
    }
    record.bpm = bpm;
    TempoChange::Again(bpm)
}

/// The notes played in time: `hits` are the times of the right notes in seconds, `slot` the
/// seconds between two notes (the beat divided by the notes per beat).
///
/// A note is in time when it comes one slot after the previous one (not a slot skipped) and its
/// place in the slot is near the average place of the notes: a steady delay of the microphone or
/// of the player is fine, drifting from the clicks is not.
pub fn notes_in_time(hits: &[f32], slot: f32) -> usize {
    if hits.is_empty() || slot <= 0.0 {
        return 0;
    }
    let tau = std::f32::consts::TAU;
    // The phase of each note in its slot, averaged on the circle.
    let phases: Vec<f32> = hits.iter().map(|t| (t / slot).rem_euclid(1.0) * tau).collect();
    let (sin, cos) = phases.iter().fold((0.0, 0.0), |(s, c), p| (s + p.sin(), c + p.cos()));
    let mean = sin.atan2(cos);
    let tolerance = (0.2 * slot).clamp(0.06, 0.12) / slot * tau;
    hits.iter()
        .zip(&phases)
        .enumerate()
        .filter(|(i, (t, phase))| {
            let off = (*phase - mean + tau / 2.0).rem_euclid(tau) - tau / 2.0;
            let next_slot = *i == 0 || ((**t - hits[i - 1]) / slot).round() == 1.0;
            off.abs() <= tolerance && next_slot
        })
        .count()
}

/// The notes of a song played in time: `hits` are the times of the right notes in seconds,
/// `expected` the times of the same notes in the score in quarter notes, `quarter` the seconds
/// of a quarter note at the tempo played. As for the drills, the place of each note is measured
/// from the player's own average delay.
pub fn notes_in_rhythm(hits: &[f32], expected: &[f32], quarter: f32) -> usize {
    let offsets: Vec<f32> = hits.iter().zip(expected).map(|(t, e)| t - e * quarter).collect();
    if offsets.is_empty() {
        return 0;
    }
    let mut sorted = offsets.clone();
    sorted.sort_by(f32::total_cmp);
    let median = sorted[sorted.len() / 2];
    // A fifth of the shortest gap between two notes, 60 to 120 ms.
    let gap = expected.windows(2).map(|w| (w[1] - w[0]) * quarter).filter(|g| *g > 0.0).fold(f32::MAX, f32::min);
    let tolerance = if gap == f32::MAX { 0.12 } else { (0.2 * gap).clamp(0.06, 0.12) };
    offsets.iter().filter(|o| (*o - median).abs() <= tolerance).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn up_to_the_target() {
        let mut r = DrillRecord::default();
        assert_eq!(tempo_of(&r), (START_BPM, DEFAULT_TARGET_BPM));
        set_tempo(&mut r, 90, 100);
        assert_eq!(after_round(&mut r, true), TempoChange::Faster(95));
        assert_eq!(after_round(&mut r, true), TempoChange::Faster(100));
        assert!(!r.target_reached);
        assert_eq!(after_round(&mut r, true), TempoChange::TargetReached(100));
        assert!(r.target_reached);
        assert_eq!(r.best_bpm, 100);
        // A higher target: on from there.
        let bpm = r.bpm;
        set_tempo(&mut r, bpm, 110);
        assert_eq!(after_round(&mut r, true), TempoChange::Faster(105));
    }

    #[test]
    fn stay_then_slow_down() {
        let mut r = DrillRecord::default();
        set_tempo(&mut r, 80, 120);
        assert_eq!(after_round(&mut r, false), TempoChange::Again(80));
        assert_eq!(after_round(&mut r, false), TempoChange::Slower(75));
        // The count starts again at the new tempo.
        assert_eq!(after_round(&mut r, false), TempoChange::Again(75));
        assert_eq!(after_round(&mut r, true), TempoChange::Faster(80));
        assert_eq!(after_round(&mut r, false), TempoChange::Again(80));
        // Never under the floor.
        set_tempo(&mut r, MIN_BPM, 120);
        after_round(&mut r, false);
        assert_eq!(after_round(&mut r, false), TempoChange::Slower(MIN_BPM));
    }

    #[test]
    fn rhythm() {
        // Quarter, eighth, eighth, half at 120 BPM (half a second a quarter), 80 ms late.
        let expected = [0.0, 1.0, 1.5, 2.0, 4.0];
        let hits: Vec<f32> = expected.iter().map(|e| 10.0 + e * 0.5 + 0.08).collect();
        assert_eq!(notes_in_rhythm(&hits, &expected, 0.5), 5);
        // The eighths played as quarters: the rest drifts.
        let even: Vec<f32> = (0..5).map(|i| 10.0 + i as f32 * 0.5).collect();
        assert!(notes_in_rhythm(&even, &expected, 0.5) < 4);
        assert_eq!(notes_in_rhythm(&[], &[], 0.5), 0);
    }

    #[test]
    fn tiers() {
        let tier = |npm| Tier::of(&DrillRecord { best_clean_npm: npm, ..Default::default() });
        assert_eq!(tier(0), None);
        assert_eq!(tier(60), Some(Tier::Basic));
        assert_eq!(tier(160), Some(Tier::Medium));
        assert_eq!(tier(300), Some(Tier::Advanced));
        assert_eq!(tier(400), Some(Tier::Legendary));
    }

    #[test]
    fn manual_tempo() {
        let mut r = DrillRecord::default();
        after_round(&mut r, false);
        assert_eq!(r.misses, 1);
        // Slower by hand: the misses start again, the target is never under the tempo.
        set_tempo(&mut r, 50, 100);
        assert_eq!(r.misses, 0);
        set_tempo(&mut r, 50, 40);
        assert_eq!(tempo_of(&r), (40, 40));
        set_tempo(&mut r, 1000, 1000);
        assert_eq!(tempo_of(&r), (MAX_BPM, MAX_BPM));
    }

    #[test]
    fn timing() {
        let slot = 0.5;
        // Steady, 70 ms after each click (the microphone): all in time.
        let steady: Vec<f32> = (0..16).map(|i| 3.0 + i as f32 * slot + 0.07).collect();
        assert_eq!(notes_in_time(&steady, slot), 16);
        // A note late by 150 ms and a slot skipped.
        let mut rough = steady.clone();
        rough[5] += 0.15;
        for t in &mut rough[10..] {
            *t += slot;
        }
        assert_eq!(notes_in_time(&rough, slot), 14);
        // Too slow for the tempo: drifting through the slots.
        let slow: Vec<f32> = (0..16).map(|i| i as f32 * 0.6).collect();
        assert!(notes_in_time(&slow, slot) < 8);
        assert_eq!(notes_in_time(&[], slot), 0);
    }
}
