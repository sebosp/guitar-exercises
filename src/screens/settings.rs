//! The settings: the instrument and its strings, the microphone with a tuner to check it, the
//! hints. Every change is saved at once.
use bevy::color::Mix;
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::audio::Mic;
use crate::music::{InstrumentKind, TuningPreset};
use crate::settings::Settings;
use crate::ui::{Btn, Fonts, Screen, button, button_frame, colors, label, screen_root, scroll_area, small_button, spawn_top_bar};

pub struct SettingsScreenPlugin;

impl Plugin for SettingsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Settings), spawn).add_systems(Update, (refresh, tuner).run_if(in_state(Screen::Settings)));
    }
}

/// A text showing a setting.
#[derive(Component)]
struct Shows(fn(&Settings) -> String);

/// A choice, highlighted when it is the setting.
#[derive(Component)]
struct Chosen(fn(&Settings) -> bool);

#[derive(Component)]
struct Tuner;

type Change = fn(&mut Settings);

fn row<'a>(parent: &'a mut ChildSpawnerCommands, name: &str, fonts: &Fonts) -> EntityCommands<'a> {
    let mut row = parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Center,
        column_gap: px(8),
        row_gap: px(6),
        ..default()
    });
    row.with_children(|r| {
        r.spawn((Node { width: px(230), ..default() }, children![label(name, fonts.text(18.0), colors::TEXT)]));
    });
    row
}

fn apply(settings: &mut Settings, change: Change) {
    change(settings);
    settings.sanitize();
    settings.save();
}

/// A row with - value +.
fn stepper(parent: &mut ChildSpawnerCommands, name: &str, fonts: &Fonts, shows: fn(&Settings) -> String, down: Change, up: Change) {
    row(parent, name, fonts).with_children(|r| {
        r.spawn(small_button("-", fonts)).observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply(&mut s, down));
        r.spawn((
            Node { min_width: px(110), justify_content: JustifyContent::Center, ..default() },
            children![(label("", fonts.bold(18.0), colors::ACCENT), Shows(shows))],
        ));
        r.spawn(small_button("+", fonts)).observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply(&mut s, up));
    });
}

/// A row of choices.
fn choices(parent: &mut ChildSpawnerCommands, name: &str, fonts: &Fonts, options: &[(&str, Chosen, Change)]) {
    row(parent, name, fonts).with_children(|r| {
        for (text, chosen, change) in options {
            let change = *change;
            r.spawn((button(*text, fonts, colors::BUTTON), Chosen(chosen.0)))
                .observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply(&mut s, change));
        }
    });
}

fn toggle(parent: &mut ChildSpawnerCommands, name: &str, fonts: &Fonts, is_on: fn(&Settings) -> bool, flip: Change) {
    row(parent, name, fonts).with_children(|r| {
        r.spawn((button_frame(colors::BUTTON), Chosen(is_on)))
            .observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply(&mut s, flip))
            .with_children(|b| {
                b.spawn((label("", fonts.text(18.0), colors::TEXT), Shows(|_| String::new())));
            });
    });
}

fn spawn(mut commands: Commands, fonts: Res<Fonts>) {
    let fonts = &*fonts;
    commands.spawn(screen_root(Screen::Settings)).with_children(|root| {
        spawn_top_bar(root, "Settings", Screen::Settings, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            list.spawn(label("Instrument", fonts.bold(20.0), colors::ACCENT));
            choices(
                list,
                "Instrument",
                fonts,
                &[
                    ("Guitar", Chosen(|s| s.instrument == InstrumentKind::Guitar), |s| switch_instrument(s, InstrumentKind::Guitar)),
                    ("Bass", Chosen(|s| s.instrument == InstrumentKind::Bass), |s| switch_instrument(s, InstrumentKind::Bass)),
                ],
            );
            stepper(
                list,
                "Strings",
                fonts,
                |s| s.strings.to_string(),
                |s| s.strings = s.strings.saturating_sub(1).max(*s.instrument.string_counts().start()),
                |s| s.strings = (s.strings + 1).min(*s.instrument.string_counts().end()),
            );
            choices(
                list,
                "Tuning",
                fonts,
                &[
                    ("Standard", Chosen(|s| s.tuning == TuningPreset::Standard), |s| s.tuning = TuningPreset::Standard),
                    ("Drop", Chosen(|s| s.tuning == TuningPreset::Drop), |s| s.tuning = TuningPreset::Drop),
                    ("Half step down", Chosen(|s| s.tuning == TuningPreset::HalfStepDown), |s| s.tuning = TuningPreset::HalfStepDown),
                ],
            );
            row(list, "Strings, low to high", fonts).with_children(|r| {
                r.spawn((
                    label("", fonts.bold(18.0), colors::ACCENT),
                    Shows(|s| {
                        let i = s.instrument();
                        (0..i.strings()).map(|n| crate::music::midi_name(i.open[n])).collect::<Vec<_>>().join("  ")
                    }),
                ));
            });
            stepper(list, "Frets", fonts, |s| s.frets.to_string(), |s| s.frets = s.frets.saturating_sub(1), |s| s.frets += 1);
            toggle(list, "Left-handed", fonts, |s| s.left_handed, |s| s.left_handed = !s.left_handed);

            list.spawn(label("Microphone", fonts.bold(20.0), colors::ACCENT));
            toggle(list, "Listen to the instrument", fonts, |s| s.microphone, |s| s.microphone = !s.microphone);
            stepper(
                list,
                "Noise gate",
                fonts,
                |s| format!("{} / 10", s.mic_gate),
                |s| s.mic_gate = s.mic_gate.saturating_sub(1),
                |s| s.mic_gate += 1,
            );
            row(list, "Tuner", fonts).with_children(|r| {
                r.spawn((label("", fonts.bold(20.0), colors::TEXT), Tuner));
            });

            list.spawn(label("Games", fonts.bold(20.0), colors::ACCENT));
            stepper(list, "Hint after", fonts, |s| format!("{:.0} s", s.hint_delay), |s| s.hint_delay -= 1.0, |s| s.hint_delay += 1.0);
            stepper(
                list,
                "Drill length",
                fonts,
                |s| format!("{} up & down", s.drill_shifts),
                |s| s.drill_shifts = s.drill_shifts.saturating_sub(1),
                |s| s.drill_shifts += 1,
            );
            toggle(list, "Note names in the drills", fonts, |s| s.show_note_names, |s| s.show_note_names = !s.show_note_names);
            toggle(list, "Sharps and flats in the note games", fonts, |s| s.accidentals, |s| s.accidentals = !s.accidentals);
            stepper(
                list,
                "Right answers to clear a note",
                fonts,
                |s| s.clear_streak.to_string(),
                |s| s.clear_streak = s.clear_streak.saturating_sub(1),
                |s| s.clear_streak += 1,
            );

            list.spawn(Node { height: px(8), ..default() });
            list.spawn(button("Reset to the defaults", fonts, colors::BUTTON))
                .insert(Node {
                    align_self: AlignSelf::Start,
                    padding: UiRect::axes(px(16), px(9)),
                    border_radius: BorderRadius::all(px(8)),
                    ..default()
                })
                .observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| {
                    apply(&mut s, |s| *s = Settings::for_instrument(s.instrument))
                });
            list.spawn(label(
                "Saved on this device. The progress is kept per instrument and number of strings.",
                fonts.text(15.0),
                colors::DIM,
            ));
        });
    });
}

/// Another instrument: its default strings, in standard tuning.
fn switch_instrument(s: &mut Settings, kind: InstrumentKind) {
    if s.instrument != kind {
        s.instrument = kind;
        s.strings = kind.default_strings();
        s.tuning = TuningPreset::Standard;
    }
}

fn refresh(
    settings: Res<Settings>,
    mut texts: Query<(&mut Text, &Shows, Option<&ChildOf>)>,
    mut choices: Query<(Entity, &Chosen, &mut Btn, &mut BackgroundColor)>,
    added: Query<(), Added<Shows>>,
) {
    if !settings.is_changed() && added.is_empty() {
        return;
    }
    let mut toggled = Vec::new();
    for (entity, chosen, mut btn, mut bg) in &mut choices {
        let on = (chosen.0)(&settings);
        btn.color = if on { colors::SELECTED } else { colors::BUTTON };
        btn.hover = btn.color.mix(&Color::WHITE, 0.12);
        bg.0 = btn.color;
        toggled.push((entity, on));
    }
    for (mut text, shows, parent) in &mut texts {
        // The text of a toggle says On or Off.
        let toggle = parent.and_then(|p| toggled.iter().find(|(e, _)| *e == p.parent()));
        let shown = match toggle {
            Some((_, on)) => if *on { "On" } else { "Off" }.to_string(),
            None => (shows.0)(&settings),
        };
        if text.0 != shown {
            text.0 = shown;
        }
    }
}

fn tuner(mic: Res<Mic>, mut texts: Query<&mut Text, With<Tuner>>) {
    if !mic.is_changed() {
        return;
    }
    let shown = match mic.pitch {
        Some(pitch) => {
            let midi = pitch.round() as i32;
            let cents = (pitch - midi as f32) * 100.0;
            format!("{}  {:+.0} cents", crate::music::midi_name(midi), cents)
        }
        None => mic.status.describe(),
    };
    for mut text in &mut texts {
        if text.0 != shown {
            text.0 = shown.clone();
        }
    }
}
