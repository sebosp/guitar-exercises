//! Which note comes next in the note games.
//!
//! "Learn notes" introduces one note at a time: its place on the staff and every place on the
//! fretboard. The new note is asked again and again, a review of an older note between two
//! questions on it, until it is answered right `clear_streak` times in a row; then the next note
//! of the learning order comes. The reviews prefer the notes with mistakes and slow answers.
use std::collections::BTreeMap;

use crate::music::Spelled;
use crate::progress::NoteStats;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Question {
    /// The new note, shown with its name and its places before it is asked.
    Introduce(usize),
    /// A note to read and play, an index in the learning order.
    Ask(usize),
}

impl Question {
    pub fn index(self) -> usize {
        match self {
            Question::Introduce(i) | Question::Ask(i) => i,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Scheduler {
    pub order: Vec<Spelled>,
    /// The notes cleared, the first `learned` of the order.
    pub learned: usize,
    pub clear_streak: u32,
    /// Right answers in a row on the new note.
    pub streak: u32,
    last: Option<usize>,
    introduced: bool,
}

impl Scheduler {
    pub fn new(order: Vec<Spelled>, learned: usize, clear_streak: u8) -> Self {
        let learned = learned.min(order.len());
        Scheduler { order, learned, clear_streak: clear_streak.max(1) as u32, streak: 0, last: None, introduced: false }
    }

    /// The note being learned, `None` once all are cleared.
    pub fn new_note(&self) -> Option<usize> {
        (self.learned < self.order.len()).then_some(self.learned)
    }

    pub fn all_learned(&self) -> bool {
        self.new_note().is_none()
    }

    pub fn next(&mut self, stats: &BTreeMap<String, NoteStats>, rng: &mut fastrand::Rng) -> Question {
        let question = match self.new_note() {
            Some(new) if !self.introduced => {
                self.introduced = true;
                Question::Introduce(new)
            }
            // Back and forth: a review after each question on the new note, when there is one.
            Some(new) if self.last == Some(new) && self.learned > 0 => Question::Ask(self.review(stats, rng, Some(new))),
            Some(new) => Question::Ask(new),
            None => Question::Ask(self.review(stats, rng, self.last)),
        };
        // The new note is asked right after its introduction.
        self.last = match question {
            Question::Introduce(_) => None,
            Question::Ask(i) => Some(i),
        };
        question
    }

    /// Records an answer; true when it cleared the new note.
    pub fn answer(&mut self, index: usize, correct: bool) -> bool {
        if Some(index) != self.new_note() {
            return false;
        }
        if !correct {
            self.streak = 0;
            return false;
        }
        self.streak += 1;
        if self.streak >= self.clear_streak {
            self.learned += 1;
            self.streak = 0;
            self.introduced = false;
            return true;
        }
        false
    }

    /// A learned note, weighted by its mistakes and its slow answers, not `avoid` when possible.
    fn review(&self, stats: &BTreeMap<String, NoteStats>, rng: &mut fastrand::Rng, avoid: Option<usize>) -> usize {
        let pool: Vec<usize> = (0..self.learned).filter(|&i| Some(i) != avoid || self.learned == 1).collect();
        if pool.is_empty() {
            return self.new_note().unwrap_or(0);
        }
        let weights: Vec<f32> = pool.iter().map(|&i| weight(stats.get(&self.order[i].full_name()))).collect();
        let total: f32 = weights.iter().sum();
        let mut pick = rng.f32() * total;
        for (i, w) in pool.iter().zip(&weights) {
            if pick < *w {
                return *i;
            }
            pick -= w;
        }
        *pool.last().unwrap_or(&0)
    }
}

/// How much a note needs a review: the mistakes, the slow answers, the notes rarely seen.
pub fn weight(stats: Option<&NoteStats>) -> f32 {
    let Some(s) = stats else {
        return 3.0;
    };
    let errors = 1.0 - s.accuracy().unwrap_or(0.5);
    let slow = (s.average_time().unwrap_or(5.0) / 3.0).min(3.0);
    let rare = 2.0 / (1.0 + s.answers() as f32).sqrt();
    0.5 + 4.0 * errors + slow + rare
}

/// A note for "Find them all": one of the learned notes (at least the first `min_pool`), not the
/// previous one.
pub fn pick_find(
    order: &[Spelled],
    learned: usize,
    min_pool: usize,
    previous: Option<usize>,
    stats: &BTreeMap<String, NoteStats>,
    rng: &mut fastrand::Rng,
) -> usize {
    let size = learned.max(min_pool).min(order.len()).max(1);
    let pool: Vec<usize> = (0..size).filter(|&i| Some(i) != previous || size == 1).collect();
    let weights: Vec<f32> = pool.iter().map(|&i| weight(stats.get(&order[i].full_name()))).collect();
    let total: f32 = weights.iter().sum();
    let mut pick = rng.f32() * total;
    for (i, w) in pool.iter().zip(&weights) {
        if pick < *w {
            return *i;
        }
        pick -= w;
    }
    pool[pool.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{Instrument, InstrumentKind, TuningPreset, learning_order};

    fn order() -> Vec<Spelled> {
        learning_order(&Instrument::new(InstrumentKind::Guitar, 6, TuningPreset::Standard, 12), false)
    }

    #[test]
    fn introduces_then_alternates() {
        let mut rng = fastrand::Rng::with_seed(7);
        let stats = BTreeMap::new();
        let mut s = Scheduler::new(order(), 0, 2);
        assert_eq!(s.next(&stats, &mut rng), Question::Introduce(0));
        // The only note: asked until cleared.
        assert_eq!(s.next(&stats, &mut rng), Question::Ask(0));
        assert!(!s.answer(0, true));
        assert_eq!(s.next(&stats, &mut rng), Question::Ask(0));
        assert!(s.answer(0, true));
        assert_eq!(s.learned, 1);
        assert_eq!(s.next(&stats, &mut rng), Question::Introduce(1));
        // New, review, new, review...
        assert_eq!(s.next(&stats, &mut rng), Question::Ask(1));
        assert_eq!(s.next(&stats, &mut rng), Question::Ask(0));
        assert_eq!(s.next(&stats, &mut rng), Question::Ask(1));
        // A mistake resets the streak.
        assert!(!s.answer(1, true));
        assert!(!s.answer(1, false));
        assert!(!s.answer(1, true));
        assert!(s.answer(1, true));
        assert_eq!(s.learned, 2);
    }

    #[test]
    fn reviews_prefer_mistakes() {
        let order = order();
        let mut stats = BTreeMap::new();
        for (i, note) in order.iter().enumerate().take(4) {
            let s: &mut NoteStats = stats.entry(note.full_name()).or_default();
            for _ in 0..10 {
                s.record(i != 2, 1.0, false);
            }
        }
        let mut rng = fastrand::Rng::with_seed(1);
        let s = Scheduler::new(order.clone(), 4, 3);
        let mut counts = [0; 4];
        for _ in 0..400 {
            counts[s.review(&stats, &mut rng, None)] += 1;
        }
        assert!(counts[2] > counts[0] * 2, "{counts:?}");
        let found = pick_find(&order, 0, 3, Some(0), &stats, &mut rng);
        assert!(found == 1 || found == 2);
    }
}
