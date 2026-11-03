//! A tuner: the note the microphone hears with a needle for the cents. In the settings it tunes
//! the strings: the needle shows how far the closest open string of the tuning is. In a drill the
//! needle shows how far the note to play is, sharp or flat, see [`Guide`].
use bevy::prelude::*;

use crate::audio::{Mic, MicStatus};
use crate::music::{midi_name, midi_to_hz};
use crate::screens::Expected;
use crate::settings::Settings;
use crate::ui::{Fonts, colors, label};

/// The cents at the ends of the needle.
const RANGE: f32 = 50.0;
/// The seconds a pitch stays shown after the sound stops.
pub const HOLD: f32 = 1.5;
/// The cents from the note within which it is in tune.
pub const IN_TUNE: f32 = 10.0;
/// The cents from the note within which it is only a bit off.
pub const CLOSE: f32 = 30.0;

/// What the tuner reads the pitch against.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Against {
    /// The closest note.
    Closest,
    /// The closest open string of the tuning, in the settings.
    Strings,
    /// The note the game waits for, in a drill.
    Target,
}

#[derive(Component)]
pub struct TunerView {
    pub against: Against,
}

/// How far a pitch is from the note to play, a semitone at most, in cents (negative: flat).
/// The octave is forgiven as the drills do, the low strings are often heard an octave off.
pub fn cents_from(pitch: f32, target: i32) -> Option<f32> {
    let diff = pitch - target as f32;
    let diff = diff - 12.0 * (diff / 12.0).round();
    (diff.abs() <= 1.0).then_some(diff * 100.0)
}

/// How a pitch sits against the note to play: a short direction to correct it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Guide {
    InTune,
    /// A bit sharp or flat: `sharp` is true when above the note.
    Close {
        sharp: bool,
    },
    /// Clearly off, up to a semitone.
    Off {
        sharp: bool,
    },
}

impl Guide {
    pub fn of(cents: f32) -> Guide {
        let sharp = cents > 0.0;
        if cents.abs() <= IN_TUNE {
            Guide::InTune
        } else if cents.abs() <= CLOSE {
            Guide::Close { sharp }
        } else {
            Guide::Off { sharp }
        }
    }

    /// The words for the player: where the pitch is and which way to go.
    pub fn words(self) -> &'static str {
        match self {
            Guide::InTune => "in tune",
            Guide::Close { sharp: true } => "a bit sharp, a little lower",
            Guide::Close { sharp: false } => "a bit flat, a little higher",
            Guide::Off { sharp: true } => "sharp, lower",
            Guide::Off { sharp: false } => "flat, higher",
        }
    }

    pub fn color(self) -> Color {
        match self {
            Guide::InTune => colors::ok(),
            Guide::Close { .. } => colors::accent(),
            Guide::Off { .. } => colors::bad(),
        }
    }
}

#[derive(Component)]
struct TunerNote;

#[derive(Component)]
struct TunerDetail;

#[derive(Component)]
struct TunerNeedle;

pub struct TunerPlugin;

impl Plugin for TunerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

pub fn spawn_tuner(parent: &mut ChildSpawnerCommands, fonts: &Fonts, against: Against) {
    parent
        .spawn((
            Node {
                width: px(260),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                padding: UiRect::all(px(10)),
                border_radius: BorderRadius::all(px(8)),
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(colors::panel()),
            TunerView { against },
            Pickable::IGNORE,
        ))
        .with_children(|t| {
            t.spawn((label("-", fonts.bold(30.0), colors::text()), TunerNote));
            t.spawn((
                Node { width: percent(100), height: px(24), border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(colors::button()),
                Pickable::IGNORE,
            ))
            .with_children(|track| {
                // In tune in the middle, a tick every 10 cents.
                for cents in (-40..=40).step_by(10) {
                    let center = cents == 0;
                    track.spawn((
                        Node {
                            position_type: PositionType::Absolute,
                            left: percent(50.0 + cents as f32 / RANGE * 50.0),
                            margin: UiRect::left(px(if center { -1.5 } else { -0.5 })),
                            width: px(if center { 3.0 } else { 1.0 }),
                            height: percent(if center { 100.0 } else { 40.0 }),
                            top: percent(if center { 0.0 } else { 30.0 }),
                            ..default()
                        },
                        BackgroundColor(if center { colors::ok() } else { colors::dim().with_alpha(0.5) }),
                        Pickable::IGNORE,
                    ));
                }
                track.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: percent(50),
                        margin: UiRect::left(px(-3)),
                        width: px(6),
                        height: percent(100),
                        border_radius: BorderRadius::all(px(3)),
                        display: Display::None,
                        ..default()
                    },
                    BackgroundColor(colors::accent()),
                    TunerNeedle,
                    Pickable::IGNORE,
                ));
            });
            t.spawn((label("", fonts.text(14.0), colors::dim()), TunerDetail));
        });
}

/// What the tuner shows: the note aimed at, the cents from it, the detail line.
#[derive(Debug, Clone, PartialEq)]
pub struct Reading {
    pub note: String,
    pub cents: f32,
    pub detail: String,
}

/// The reading of a pitch (a fractional MIDI number): to the closest note, or to the closest
/// open string of `open`, numbered from the highest as the guitarists do.
pub fn read(pitch: f32, open: Option<&[i32]>) -> Reading {
    let hz = midi_to_hz(pitch);
    let heard = pitch.round() as i32;
    match open.filter(|o| !o.is_empty()) {
        None => {
            let cents = (pitch - heard as f32) * 100.0;
            Reading { note: midi_name(heard), cents, detail: format!("{hz:.1} Hz, {cents:+.0} cents") }
        }
        Some(open) => read_strings(pitch, open),
    }
}

/// The reading of a pitch against the note to play: within a semitone the cents and the way to
/// go, further the note heard instead.
pub fn read_target(pitch: f32, target: i32) -> Reading {
    let hz = midi_to_hz(pitch);
    match cents_from(pitch, target) {
        Some(cents) => {
            Reading { note: midi_name(target), cents, detail: format!("{hz:.1} Hz, {cents:+.0} cents: {}", Guide::of(cents).words()) }
        }
        None => read(pitch, None),
    }
}

/// To the closest open string, numbered from the highest as the guitarists do.
fn read_strings(pitch: f32, open: &[i32]) -> Reading {
    let hz = midi_to_hz(pitch);
    let heard = pitch.round() as i32;
    let (string, &target) =
        open.iter().enumerate().min_by(|a, b| (pitch - *a.1 as f32).abs().total_cmp(&(pitch - *b.1 as f32).abs())).expect("strings");
    let cents = (pitch - target as f32) * 100.0;
    let advice = if cents.abs() <= 5.0 {
        "in tune"
    } else if cents < 0.0 {
        "tune up"
    } else {
        "tune down"
    };
    Reading {
        note: format!("String {}: {}", open.len() - string, midi_name(target)),
        cents,
        detail: format!("Heard {} at {hz:.1} Hz, {cents:+.0} cents: {advice}", midi_name(heard)),
    }
}

#[allow(clippy::too_many_arguments)]
fn update(
    mic: Res<Mic>,
    settings: Res<Settings>,
    expected: Res<Expected>,
    time: Res<Time<Real>>,
    mut smoothed: Local<Option<f32>>,
    mut heard_at: Local<f32>,
    views: Query<&TunerView>,
    mut notes: Query<(&mut Text, &mut TextColor), (With<TunerNote>, Without<TunerDetail>)>,
    mut details: Query<&mut Text, (With<TunerDetail>, Without<TunerNote>)>,
    mut needles: Query<(&mut Node, &mut BackgroundColor), With<TunerNeedle>>,
) {
    let Some(view) = views.iter().next() else {
        *smoothed = None;
        return;
    };
    let now = time.elapsed_secs();
    // A light smoothing of the needle, reset on another note.
    if let Some(pitch) = mic.pitch {
        *smoothed = Some(match *smoothed {
            Some(s) if (s - pitch).abs() < 0.5 => s + 0.35 * (pitch - s),
            _ => pitch,
        });
        *heard_at = now;
    } else if now - *heard_at > HOLD {
        *smoothed = None;
    }
    let open = settings.instrument().open;
    let reading = smoothed.map(|p| match (view.against, expected.midi) {
        (Against::Strings, _) => read(p, Some(open.as_slice())),
        (Against::Target, Some(target)) => read_target(p, target),
        _ => read(p, None),
    });
    let fresh = mic.pitch.is_some();
    let (note, detail, color) = match (&reading, &mic.status) {
        (Some(r), _) => {
            let color = Guide::of(r.cents).color();
            (r.note.clone(), r.detail.clone(), if fresh { color } else { color.with_alpha(0.5) })
        }
        (None, MicStatus::Listening) => ("-".into(), "Play a note".into(), colors::dim()),
        (None, status) => ("-".into(), status.describe(), colors::dim()),
    };
    for (mut text, mut text_color) in &mut notes {
        if text.0 != note {
            text.0 = note.clone();
        }
        text_color.0 = if reading.is_some() { colors::text() } else { colors::dim() };
    }
    for mut text in &mut details {
        if text.0 != detail {
            text.0 = detail.clone();
        }
    }
    for (mut node, mut bg) in &mut needles {
        match &reading {
            Some(r) => {
                node.display = Display::Flex;
                node.left = percent(50.0 + r.cents.clamp(-RANGE, RANGE) / RANGE * 50.0);
                bg.0 = color;
            }
            None => node.display = Display::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readings() {
        let a = read(45.12, None);
        assert_eq!(a.note, "A2");
        assert!((a.cents - 12.0).abs() < 0.01);
        assert!(a.detail.starts_with("110.8 Hz"));
        // A 5-string bass with the low string down to A0, the B still there: tune down.
        let open = [21, 28, 33, 38, 43];
        let b = read(23.0, Some(&open));
        assert_eq!(b.note, "String 5: A0");
        assert!((b.cents - 200.0).abs() < 0.01);
        assert!(b.detail.contains("tune down"));
        assert_eq!(read(27.6, Some(&open)).note, "String 4: E1");
        assert!(read(27.6, Some(&open)).detail.contains("tune up"));
        assert!(read(42.98, Some(&open)).detail.contains("in tune"));
    }

    #[test]
    fn against_the_target() {
        // A fretless note halfway to the next fret: sharp, come down.
        let r = read_target(45.5, 45);
        assert_eq!(r.note, "A2");
        assert!((r.cents - 50.0).abs() < 0.01);
        assert!(r.detail.ends_with("sharp, lower"));
        assert_eq!(Guide::of(-20.0), Guide::Close { sharp: false });
        assert_eq!(Guide::of(-20.0).words(), "a bit flat, a little higher");
        assert_eq!(Guide::of(4.0), Guide::InTune);
        // An octave off is forgiven, as the drills do; two semitones off is another note.
        assert!((cents_from(57.1, 45).expect("an octave up") - 10.0).abs() < 0.01);
        assert_eq!(cents_from(47.0, 45), None);
        assert_eq!(read_target(47.0, 45).note, "B2");
    }
}
