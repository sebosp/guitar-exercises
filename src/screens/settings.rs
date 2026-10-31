//! The settings: the instrument and its strings, the microphone with a tuner to check it, the
//! hints. Every change is saved at once.
use bevy::color::Mix;
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::music::{InstrumentKind, RETUNE_RANGE, TuningPreset, midi_name};
use crate::settings::Settings;
use crate::theme::Theme;
use crate::ui::{Btn, Fonts, Screen, button, button_frame, colors, label, screen_root, scroll_area, small_button, spawn_top_bar};
use crate::zoom::Zoom;

pub struct SettingsScreenPlugin;

impl Plugin for SettingsScreenPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Settings), spawn)
            .add_systems(Update, (refresh, refresh_strings, show_zoom).run_if(in_state(Screen::Settings)));
    }
}

/// A text showing a setting.
#[derive(Component)]
struct Shows(fn(&Settings) -> String);

/// A choice, highlighted when it is the setting.
#[derive(Component)]
struct Chosen(fn(&Settings) -> bool);

/// The row of a string's tuning, hidden when the instrument has fewer strings; 0 is the lowest.
#[derive(Component)]
struct StringRow(usize);

/// The name of a string in its row.
#[derive(Component)]
struct StringName(usize);

/// The note of a string and its offset from the standard tuning.
#[derive(Component)]
struct StringNote(usize);

/// How to zoom and go full screen on this platform.
const ZOOM_HELP: &str = if cfg!(target_os = "android") {
    "Or two fingers on the screen, apart or together."
} else if cfg!(target_arch = "wasm32") {
    "Or two fingers on a touch screen. The browser zooms the whole page with Ctrl, and F11 is its full screen."
} else if cfg!(target_os = "macos") {
    "Or Cmd and the mouse wheel, a pinch of the trackpad, Cmd+= Cmd+- Cmd+0. Cmd+Ctrl+F or F11: full screen, to read it from the instrument."
} else {
    "Or Ctrl and the mouse wheel, a pinch of the trackpad, two fingers on a touch screen, Ctrl+= Ctrl+- Ctrl+0. F11: full screen, to read it from the instrument."
};

/// The zoom in percent.
#[derive(Component)]
struct ZoomText;

/// A theme button, highlighted when it is the theme.
#[derive(Component)]
struct ThemeChoice(Theme);

/// The most strings of an instrument, the rows spawned.
const MAX_STRINGS: usize = 8;

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
        r.spawn((Node { width: px(230), ..default() }, children![label(name, fonts.text(18.0), colors::text())]));
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
            children![(label("", fonts.bold(18.0), colors::accent()), Shows(shows))],
        ));
        r.spawn(small_button("+", fonts)).observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply(&mut s, up));
    });
}

/// A row of choices.
fn choices(parent: &mut ChildSpawnerCommands, name: &str, fonts: &Fonts, options: &[(&str, Chosen, Change)]) {
    row(parent, name, fonts).with_children(|r| {
        for (text, chosen, change) in options {
            let change = *change;
            r.spawn((button(*text, fonts, colors::button()), Chosen(chosen.0)))
                .observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply(&mut s, change));
        }
    });
}

fn toggle(parent: &mut ChildSpawnerCommands, name: &str, fonts: &Fonts, is_on: fn(&Settings) -> bool, flip: Change) {
    row(parent, name, fonts).with_children(|r| {
        r.spawn((button_frame(colors::button()), Chosen(is_on)))
            .observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply(&mut s, flip))
            .with_children(|b| {
                b.spawn((label("", fonts.text(18.0), colors::text()), Shows(|_| String::new())));
            });
    });
}

fn spawn(mut commands: Commands, fonts: Res<Fonts>) {
    let fonts = &*fonts;
    commands.spawn(screen_root(Screen::Settings)).with_children(|root| {
        spawn_top_bar(root, "Settings", Screen::Settings, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            list.spawn(label("Instrument", fonts.bold(20.0), colors::accent()));
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
                // Another number of strings starts in standard tuning.
                |s| {
                    s.strings = s.strings.saturating_sub(1).max(*s.instrument.string_counts().start());
                    s.tuning.clear();
                },
                |s| {
                    s.strings = (s.strings + 1).min(*s.instrument.string_counts().end());
                    s.tuning.clear();
                },
            );
            choices(
                list,
                "Tuning",
                fonts,
                &[
                    ("Standard", Chosen(|s| TuningPreset::matching(&s.tuning) == Some(TuningPreset::Standard)), |s| {
                        s.tuning = TuningPreset::Standard.offsets(s.strings)
                    }),
                    ("Drop", Chosen(|s| TuningPreset::matching(&s.tuning) == Some(TuningPreset::Drop)), |s| {
                        s.tuning = TuningPreset::Drop.offsets(s.strings)
                    }),
                    ("Half step down", Chosen(|s| TuningPreset::matching(&s.tuning) == Some(TuningPreset::HalfStepDown)), |s| {
                        s.tuning = TuningPreset::HalfStepDown.offsets(s.strings)
                    }),
                ],
            );
            // One row per string, the highest on top as on the fretboard.
            for string in (0..MAX_STRINGS).rev() {
                string_row(list, string, fonts);
            }
            row(list, "Strings, low to high", fonts).with_children(|r| {
                r.spawn((
                    label("", fonts.bold(18.0), colors::accent()),
                    Shows(|s| {
                        let i = s.instrument();
                        (0..i.strings()).map(|n| midi_name(i.open[n])).collect::<Vec<_>>().join("  ")
                    }),
                ));
            });
            stepper(list, "Frets", fonts, |s| s.frets.to_string(), |s| s.frets = s.frets.saturating_sub(1), |s| s.frets += 1);
            toggle(list, "Left-handed", fonts, |s| s.left_handed, |s| s.left_handed = !s.left_handed);

            list.spawn(label("Look", fonts.bold(20.0), colors::accent()));
            row(list, "Theme", fonts).with_children(|r| {
                for theme in Theme::ALL {
                    r.spawn((button(theme.name(), fonts, colors::button()), ThemeChoice(theme))).observe(
                        move |_: On<PointerClick>, mut s: ResMut<Settings>| {
                            s.theme = theme;
                            s.save();
                        },
                    );
                }
            });

            row(list, "Zoom", fonts).with_children(|r| {
                r.spawn(small_button("-", fonts)).observe(|_: On<PointerClick>, mut zoom: MessageWriter<Zoom>| {
                    zoom.write(Zoom::OUT);
                });
                r.spawn((
                    Node { min_width: px(70), justify_content: JustifyContent::Center, ..default() },
                    children![(label("", fonts.bold(18.0), colors::accent()), ZoomText)],
                ));
                r.spawn(small_button("+", fonts)).observe(|_: On<PointerClick>, mut zoom: MessageWriter<Zoom>| {
                    zoom.write(Zoom::IN);
                });
                r.spawn(button("Fit the window", fonts, colors::button())).observe(|_: On<PointerClick>, mut zoom: MessageWriter<Zoom>| {
                    zoom.write(Zoom::Reset);
                });
            });
            list.spawn(label(ZOOM_HELP, fonts.text(15.0), colors::dim()));

            list.spawn(label("Microphone", fonts.bold(20.0), colors::accent()));
            toggle(list, "Listen to the instrument", fonts, |s| s.microphone, |s| s.microphone = !s.microphone);
            stepper(
                list,
                "Noise gate",
                fonts,
                |s| format!("{} / 10", s.mic_gate),
                |s| s.mic_gate = s.mic_gate.saturating_sub(1),
                |s| s.mic_gate += 1,
            );
            row(list, "Tuner, to the strings", fonts).with_children(|r| crate::tuner::spawn_tuner(r, fonts, true));

            list.spawn(label("Games", fonts.bold(20.0), colors::accent()));
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
            list.spawn(button("Reset to the defaults", fonts, colors::button()))
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
                colors::dim(),
            ));
        });
    });
}

/// Another instrument: its default strings, in standard tuning.
fn switch_instrument(s: &mut Settings, kind: InstrumentKind) {
    if s.instrument != kind {
        s.instrument = kind;
        s.strings = kind.default_strings();
        s.tuning.clear();
    }
}

/// A string a semitone up or down.
fn retune(s: &mut Settings, string: usize, delta: i8) {
    if let Some(offset) = s.tuning.get_mut(string) {
        *offset = (*offset + delta).clamp(*RETUNE_RANGE.start(), *RETUNE_RANGE.end());
    }
}

/// - note +, for one string.
fn string_row(parent: &mut ChildSpawnerCommands, string: usize, fonts: &Fonts) {
    parent
        .spawn((
            Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(8), ..default() },
            StringRow(string),
        ))
        .with_children(|r| {
            r.spawn((Node { width: px(230), ..default() }, children![(label("", fonts.text(18.0), colors::text()), StringName(string))]));
            r.spawn(small_button("-", fonts)).observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply_retune(&mut s, string, -1));
            r.spawn((
                Node { min_width: px(110), justify_content: JustifyContent::Center, ..default() },
                children![(label("", fonts.bold(18.0), colors::accent()), StringNote(string))],
            ));
            r.spawn(small_button("+", fonts)).observe(move |_: On<PointerClick>, mut s: ResMut<Settings>| apply_retune(&mut s, string, 1));
        });
}

fn apply_retune(settings: &mut Settings, string: usize, delta: i8) {
    retune(settings, string, delta);
    settings.sanitize();
    settings.save();
}

/// The string rows: hidden past the strings of the instrument, the names and the notes.
fn refresh_strings(
    settings: Res<Settings>,
    mut rows: Query<(&StringRow, &mut Node)>,
    mut names: Query<(&StringName, &mut Text), Without<StringNote>>,
    mut notes: Query<(&StringNote, &mut Text, &mut TextColor), Without<StringName>>,
    added: Query<(), Added<StringRow>>,
) {
    if !settings.is_changed() && added.is_empty() {
        return;
    }
    let instrument = settings.instrument();
    let strings = instrument.strings();
    for (row, mut node) in &mut rows {
        let display = if row.0 < strings { Display::Flex } else { Display::None };
        if node.display != display {
            node.display = display;
        }
    }
    for (name, mut text) in &mut names {
        if name.0 >= strings {
            continue;
        }
        // String 1 is the highest, as the guitarists count.
        let number = strings - name.0;
        let place = match name.0 {
            0 => " (lowest)",
            n if n == strings - 1 => " (highest)",
            _ => "",
        };
        text.0 = format!("String {number}{place}");
    }
    for (note, mut text, mut color) in &mut notes {
        let Some(&open) = instrument.open.get(note.0) else {
            continue;
        };
        let offset = settings.tuning.get(note.0).copied().unwrap_or(0);
        text.0 = if offset == 0 { midi_name(open) } else { format!("{}  ({offset:+})", midi_name(open)) };
        color.0 = if offset == 0 { colors::accent() } else { colors::reveal() };
    }
}

fn refresh(
    settings: Res<Settings>,
    mut texts: Query<(&mut Text, &Shows, Option<&ChildOf>)>,
    mut choices: Query<(Entity, &Chosen, &mut Btn, &mut BackgroundColor), Without<ThemeChoice>>,
    mut themes: Query<(&ThemeChoice, &mut Btn, &mut BackgroundColor), Without<Chosen>>,
    added: Query<(), Added<Shows>>,
) {
    if !settings.is_changed() && added.is_empty() {
        return;
    }
    let mut toggled = Vec::new();
    for (theme, mut btn, mut bg) in &mut themes {
        btn.color = if theme.0 == settings.theme { colors::selected() } else { colors::button() };
        btn.hover = btn.color.mix(&Color::WHITE, 0.12);
        bg.0 = btn.color;
    }
    for (entity, chosen, mut btn, mut bg) in &mut choices {
        let on = (chosen.0)(&settings);
        btn.color = if on { colors::selected() } else { colors::button() };
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

fn show_zoom(scale: Res<UiScale>, mut texts: Query<&mut Text, With<ZoomText>>) {
    let shown = crate::zoom::percent(scale.0);
    for mut text in &mut texts {
        if text.0 != shown {
            text.0 = shown.clone();
        }
    }
}
