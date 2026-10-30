//! Pitch detection: the YIN estimator on a mono buffer, and a tracker that turns the pitches
//! into played notes (onsets).
//!
//! The buffers are decimated to about 12 kHz first: the fundamentals of a guitar and a bass are
//! under 1.4 kHz, and YIN is quadratic in the window size.

/// The lowest note listened for, a bit under B0 (30.9 Hz) of a 5-string bass.
pub const MIN_HZ: f32 = 28.0;
pub const MAX_HZ: f32 = 1400.0;
const TARGET_RATE: f32 = 12_000.0;

/// Averages groups of samples: a box low pass and a decimation, down to about 12 kHz.
pub fn decimate(samples: &[f32], sample_rate: f32) -> (Vec<f32>, f32) {
    let factor = ((sample_rate / TARGET_RATE).floor() as usize).max(1);
    if factor == 1 {
        return (samples.to_vec(), sample_rate);
    }
    let out = samples.chunks_exact(factor).map(|c| c.iter().sum::<f32>() / factor as f32).collect();
    (out, sample_rate / factor as f32)
}

pub fn rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt()
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pitch {
    pub hz: f32,
    /// 1 for a pure periodic signal, 0 for noise.
    pub clarity: f32,
}

/// The YIN estimator (de Cheveigné and Kawahara, 2002): the first dip of the cumulative mean
/// normalized difference under the threshold, refined with a parabola.
pub fn yin(samples: &[f32], sample_rate: f32, min_hz: f32, max_hz: f32) -> Option<Pitch> {
    const THRESHOLD: f32 = 0.15;
    let tau_min = ((sample_rate / max_hz).floor() as usize).max(2);
    let tau_max = (sample_rate / min_hz).ceil() as usize;
    if samples.len() < 2 * tau_max || tau_max <= tau_min {
        return None;
    }
    let window = samples.len() - tau_max;
    let mut diff = vec![0.0f32; tau_max + 1];
    for (tau, d) in diff.iter_mut().enumerate().skip(1) {
        let mut sum = 0.0;
        for i in 0..window {
            let delta = samples[i] - samples[i + tau];
            sum += delta * delta;
        }
        *d = sum;
    }
    // Cumulative mean normalized difference.
    let mut cmnd = vec![1.0f32; tau_max + 1];
    let mut running = 0.0;
    for tau in 1..=tau_max {
        running += diff[tau];
        cmnd[tau] = if running > 0.0 { diff[tau] * tau as f32 / running } else { 1.0 };
    }
    let mut tau = tau_min;
    let mut found = None;
    while tau < tau_max {
        if cmnd[tau] < THRESHOLD {
            // Down to the bottom of the dip.
            while tau + 1 < tau_max && cmnd[tau + 1] < cmnd[tau] {
                tau += 1;
            }
            found = Some(tau);
            break;
        }
        tau += 1;
    }
    let tau = found?;
    let better = if tau > 1 && tau < tau_max {
        let (a, b, c) = (cmnd[tau - 1], cmnd[tau], cmnd[tau + 1]);
        let denom = a - 2.0 * b + c;
        if denom.abs() > f32::EPSILON { tau as f32 + 0.5 * (a - c) / denom } else { tau as f32 }
    } else {
        tau as f32
    };
    Some(Pitch { hz: sample_rate / better, clarity: 1.0 - cmnd[tau] })
}

/// A note played: its MIDI number and how far from it, in cents.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Onset {
    pub midi: i32,
    pub cents: f32,
}

/// Turns the pitch of each analysis into the notes played: a note counts once it holds for a few
/// analyses, and again after silence, after another note, or when the level jumps (a new pluck).
#[derive(Debug, Clone)]
pub struct NoteTracker {
    /// Under this level the instrument is silent.
    pub gate: f32,
    candidate: Option<i32>,
    count: u32,
    current: Option<i32>,
    /// The lowest level since the last onset, to see a new pluck of the same note.
    valley: f32,
}

/// The analyses a pitch must hold to be a note, about 50 ms at 60 analyses per second.
const STABLE: u32 = 3;

impl NoteTracker {
    pub fn new(gate: f32) -> Self {
        NoteTracker { gate, candidate: None, count: 0, current: None, valley: f32::MAX }
    }

    /// The note sounding now, if any.
    pub fn current(&self) -> Option<i32> {
        self.current
    }

    pub fn feed(&mut self, midi: Option<f32>, level: f32) -> Option<Onset> {
        let Some(midi) = midi.filter(|_| level >= self.gate) else {
            self.candidate = None;
            self.count = 0;
            self.current = None;
            self.valley = f32::MAX;
            return None;
        };
        let rounded = midi.round() as i32;
        // A new pluck of the ringing note: the level jumps over the valley since the onset.
        if self.current == Some(rounded) && level > 2.0 * self.valley {
            self.current = None;
            self.count = 0;
        }
        self.valley = self.valley.min(level);
        if self.candidate == Some(rounded) {
            self.count += 1;
        } else {
            self.candidate = Some(rounded);
            self.count = 1;
        }
        if self.count >= STABLE && self.current != Some(rounded) {
            self.current = Some(rounded);
            self.valley = level;
            return Some(Onset { midi: rounded, cents: (midi - rounded as f32) * 100.0 });
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music::{hz_to_midi, midi_to_hz};

    /// A plucked string: a fundamental and decaying harmonics.
    fn pluck(hz: f32, rate: f32, len: usize) -> Vec<f32> {
        (0..len)
            .map(|i| {
                let t = i as f32 / rate;
                (1..=5).map(|h| (2.0 * std::f32::consts::PI * hz * h as f32 * t).sin() * 0.5 / h as f32).sum::<f32>()
            })
            .collect()
    }

    #[test]
    fn detects_guitar_and_bass_notes() {
        let rate = 48_000.0;
        // B0 of a 5-string bass, E1, E2, A2, the open high e, fret 12 of the high e.
        for midi in [23, 28, 40, 45, 64, 76] {
            let hz = midi_to_hz(midi as f32);
            let raw = pluck(hz, rate, 8192);
            let (samples, sr) = decimate(&raw, rate);
            let pitch = yin(&samples, sr, MIN_HZ, MAX_HZ).expect("a pitch");
            let found = hz_to_midi(pitch.hz);
            assert!((found - midi as f32).abs() < 0.3, "midi {midi}: found {found}");
            assert!(pitch.clarity > 0.8);
        }
    }

    #[test]
    fn noise_has_no_pitch() {
        let mut state = 12345u32;
        let noise: Vec<f32> = (0..4096)
            .map(|_| {
                state = state.wrapping_mul(1_103_515_245).wrapping_add(12345);
                (state >> 16) as f32 / 32768.0 - 1.0
            })
            .collect();
        let pitch = yin(&noise, 12_000.0, MIN_HZ, MAX_HZ);
        assert!(pitch.is_none_or(|p| p.clarity < 0.9));
    }

    #[test]
    fn tracker_onsets() {
        let mut t = NoteTracker::new(0.01);
        // Silence, then E2 held: one onset after it is stable.
        assert_eq!(t.feed(None, 0.0), None);
        assert_eq!(t.feed(Some(40.1), 0.2), None);
        assert_eq!(t.feed(Some(40.0), 0.2), None);
        assert_eq!(t.feed(Some(39.9), 0.2).map(|o| o.midi), Some(40));
        assert_eq!(t.feed(Some(40.0), 0.15), None);
        // Another note.
        for _ in 0..2 {
            assert_eq!(t.feed(Some(41.0), 0.15), None);
        }
        assert_eq!(t.feed(Some(41.0), 0.15).map(|o| o.midi), Some(41));
        // Decays, then the same note plucked again.
        t.feed(Some(41.0), 0.05);
        assert_eq!(t.feed(Some(41.0), 0.3), None);
        t.feed(Some(41.0), 0.3);
        assert_eq!(t.feed(Some(41.0), 0.3).map(|o| o.midi), Some(41));
        // Under the gate: nothing.
        assert_eq!(t.feed(Some(50.0), 0.001), None);
        assert_eq!(t.current(), None);
    }
}
