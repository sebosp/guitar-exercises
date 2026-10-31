//! A tuner: the note the microphone hears with a needle for the cents. In the settings it tunes
//! the strings: the needle shows how far the closest open string of the tuning is.
use bevy::prelude::*;

use crate::audio::{Mic, MicStatus};
use crate::music::{midi_name, midi_to_hz};
use crate::settings::Settings;
use crate::ui::{Fonts, colors, label};

/// The cents at the ends of the needle.
const RANGE: f32 = 50.0;
/// The seconds a pitch stays shown after the sound stops.
const HOLD: f32 = 1.5;

#[derive(Component)]
pub struct TunerView {
    /// To the open strings instead of the closest note.
    pub strings: bool,
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

pub fn spawn_tuner(parent: &mut ChildSpawnerCommands, fonts: &Fonts, strings: bool) {
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
            BackgroundColor(colors::PANEL),
            TunerView { strings },
            Pickable::IGNORE,
        ))
        .with_children(|t| {
            t.spawn((label("-", fonts.bold(30.0), colors::TEXT), TunerNote));
            t.spawn((
                Node { width: percent(100), height: px(24), border_radius: BorderRadius::all(px(4)), ..default() },
                BackgroundColor(colors::BUTTON),
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
                        BackgroundColor(if center { colors::OK } else { colors::DIM.with_alpha(0.5) }),
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
                    BackgroundColor(colors::ACCENT),
                    TunerNeedle,
                    Pickable::IGNORE,
                ));
            });
            t.spawn((label("", fonts.text(14.0), colors::DIM), TunerDetail));
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
            Reading { note: midi_name(heard), cents, detail: format!("{hz:.1} Hz · {cents:+.0} cents") }
        }
        Some(open) => {
            let (string, &target) = open
                .iter()
                .enumerate()
                .min_by(|a, b| (pitch - *a.1 as f32).abs().total_cmp(&(pitch - *b.1 as f32).abs()))
                .expect("strings");
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
                detail: format!("Heard {} {hz:.1} Hz · {cents:+.0} cents, {advice}", midi_name(heard)),
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn update(
    mic: Res<Mic>,
    settings: Res<Settings>,
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
    let reading = smoothed.map(|p| read(p, view.strings.then_some(open.as_slice())));
    let fresh = mic.pitch.is_some();
    let (note, detail, color) = match (&reading, &mic.status) {
        (Some(r), _) => {
            let color = if r.cents.abs() <= 5.0 {
                colors::OK
            } else if r.cents.abs() <= 15.0 {
                colors::ACCENT
            } else {
                colors::BAD
            };
            (r.note.clone(), r.detail.clone(), if fresh { color } else { color.with_alpha(0.5) })
        }
        (None, MicStatus::Listening) => ("-".into(), "Play a note".into(), colors::DIM),
        (None, status) => ("-".into(), status.describe(), colors::DIM),
    };
    for (mut text, mut text_color) in &mut notes {
        if text.0 != note {
            text.0 = note.clone();
        }
        text_color.0 = if reading.is_some() { colors::TEXT } else { colors::DIM };
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
}
