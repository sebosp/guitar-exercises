//! The practice by day, as in claudralingo: the minutes played (counted between the notes, a
//! pause of more than half a minute is not practice), the rounds and the rounds with a star, and
//! every round of a drill as a session. The streak is the days in a row with the daily goal of
//! minutes met. Saved with the progress.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::audio::NoteHeard;
use crate::fretboard::FretTapped;
use crate::progress::Progress;
use crate::storage::now_unix_seconds;
use crate::ui::Screen;

pub const SECONDS_PER_DAY: u64 = 86_400;
/// A pause longer than this between two notes is not practice.
const IDLE_SECONDS: f32 = 30.0;
/// The sessions kept, the oldest go first.
const MAX_SESSIONS: usize = 5000;
/// The practice not saved yet, in seconds, before a save.
const SAVE_EVERY: f32 = 20.0;

/// What a day holds.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Day {
    pub seconds: f32,
    /// The notes played or tapped, right or wrong.
    pub notes: u32,
    pub rounds: u32,
    /// The rounds with a star at least.
    pub starred: u32,
}

/// A round of a drill.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Session {
    /// Seconds since the UNIX epoch.
    pub finished_at: u64,
    pub key: String,
    /// The section of the exercise, `Finger drills`, `Scales`...
    pub kind: String,
    pub accuracy: f32,
    pub notes_per_minute: f32,
    /// The share of the notes in time with the metronome.
    pub in_time: Option<f32>,
    pub seconds: f32,
}

/// The days since the UNIX epoch, in UTC.
pub fn day_of(seconds: u64) -> u64 {
    seconds / SECONDS_PER_DAY
}

pub fn today() -> u64 {
    day_of(now_unix_seconds())
}

/// The days in a row up to today with the daily goal met, today counting when met.
pub fn streak(days: &std::collections::BTreeMap<u64, Day>, today: u64, goal_minutes: u32) -> usize {
    let met = |d: u64| days.get(&d).is_some_and(|day| day.seconds >= goal_minutes as f32 * 60.0);
    let mut day = if met(today) { today } else { today.saturating_sub(1) };
    let mut streak = 0;
    while met(day) {
        streak += 1;
        if day == 0 {
            break;
        }
        day -= 1;
    }
    streak
}

/// The year, the month (1 to 12) and the day (1 to 31) of a day since the UNIX epoch, the
/// civil calendar of Howard Hinnant.
pub fn civil(days: u64) -> (i64, u32, u32) {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d)
}

/// The day since the UNIX epoch of a date.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> u64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let m = m as i64;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe - 719_468) as u64
}

pub const MONTHS: [&str; 12] =
    ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

/// `Oct 27` of a day.
pub fn short_date(day: u64) -> String {
    let (_, m, d) = civil(day);
    format!("{} {d}", &MONTHS[m as usize - 1][..3])
}

impl Progress {
    pub fn day(&mut self, day: u64) -> &mut Day {
        self.days.entry(day).or_default()
    }

    /// A round finished: a session, and the rounds of the day.
    pub fn log_round(&mut self, session: Session, starred: bool) {
        let day = self.day(day_of(session.finished_at));
        day.rounds += 1;
        day.starred += u32::from(starred);
        self.sessions.push(session);
        let excess = self.sessions.len().saturating_sub(MAX_SESSIONS);
        self.sessions.drain(..excess);
    }
}

pub struct PracticePlugin;

impl Plugin for PracticePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, count_practice)
            .add_systems(OnExit(Screen::Drill), save)
            .add_systems(OnExit(Screen::Learn), save)
            .add_systems(OnExit(Screen::Find), save);
    }
}

/// The time between two notes of a game, when short enough, is practice.
fn count_practice(
    mut heard: MessageReader<NoteHeard>,
    mut tapped: MessageReader<FretTapped>,
    screen: Res<State<Screen>>,
    time: Res<Time<Real>>,
    mut last: Local<Option<f32>>,
    mut unsaved: Local<f32>,
    mut progress: ResMut<Progress>,
) {
    let notes = heard.read().count() + tapped.read().count();
    if notes == 0 || !matches!(screen.get(), Screen::Drill | Screen::Learn | Screen::Find) {
        return;
    }
    let now = time.elapsed_secs();
    let gap = last.map_or(0.0, |l| now - l);
    *last = Some(now);
    let played = if gap <= IDLE_SECONDS { gap } else { 0.0 };
    let day = progress.day(today());
    day.seconds += played;
    day.notes += notes as u32;
    *unsaved += played;
    if *unsaved >= SAVE_EVERY {
        *unsaved = 0.0;
        progress.save();
    }
}

fn save(progress: Res<Progress>) {
    progress.save();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    #[test]
    fn calendar() {
        assert_eq!(civil(0), (1970, 1, 1));
        assert_eq!(civil(days_from_civil(2026, 10, 31)), (2026, 10, 31));
        assert_eq!(civil(days_from_civil(2024, 2, 29)), (2024, 2, 29));
        assert_eq!(short_date(days_from_civil(2026, 10, 27)), "Oct 27");
    }

    #[test]
    fn streaks() {
        let today = 20_000;
        let minutes = |m: f32| Day { seconds: m * 60.0, ..Default::default() };
        let mut days: BTreeMap<u64, Day> = BTreeMap::new();
        assert_eq!(streak(&days, today, 10), 0);
        days.insert(today - 1, minutes(12.0));
        days.insert(today - 2, minutes(10.0));
        days.insert(today - 3, minutes(4.0));
        // Today not met yet: the streak of the days before still counts.
        assert_eq!(streak(&days, today, 10), 2);
        days.insert(today, minutes(10.5));
        assert_eq!(streak(&days, today, 10), 3);
        assert_eq!(streak(&days, today, 3), 4);
    }

    #[test]
    fn rounds() {
        let mut p = Progress::default();
        p.log_round(Session { finished_at: 3 * SECONDS_PER_DAY + 5, accuracy: 0.9, ..Default::default() }, true);
        p.log_round(Session { finished_at: 3 * SECONDS_PER_DAY + 9, accuracy: 0.5, ..Default::default() }, false);
        assert_eq!(p.days[&3], Day { rounds: 2, starred: 1, ..Default::default() });
        assert_eq!(p.sessions.len(), 2);
    }
}
