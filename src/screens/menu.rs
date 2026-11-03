//! The main menu, a setlist: the tuning of the instrument drawn as its strings, then the games,
//! the statistics and the settings in rows, each with where the player is in it.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::exercises::Page;
use crate::music::{TuningPreset, learning_order};
use crate::progress::{InstrumentProgress, Progress};
use crate::settings::Settings;
use crate::ui::{Fonts, Screen, button_frame, colors, label, screen_root, scroll_area, size, spawn_top_bar};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Menu), spawn);
    }
}

const ENTRIES: [(&str, &str, Screen); 8] = [
    ("Finger drills", "The four fingers in every order, wider spans, string crossings, stretches", Screen::Drills),
    ("Scales", "Any scale and mode as runs, sequences and the notes in every order", Screen::Scales),
    ("Songs", "Your Guitar Pro files, a section at a time, slow to full speed", Screen::Songs),
    ("Learn notes", "Read a note on the staff, find it on the fretboard, play it", Screen::Learn),
    ("Find them all", "Tap every place of a note on the fretboard", Screen::Find),
    ("Statistics", "Your days, your streak, your speed and accuracy", Screen::Stats),
    ("Settings", "Instrument, tuning, microphone, look", Screen::Settings),
    ("Help", "Stars, tempo, tiers and keys, F1", Screen::Help),
];

/// The widest the setlist gets: a line of text stays short enough to read.
const COLUMN: f32 = 760.0;

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>, progress: Res<Progress>) {
    let fonts = &*fonts;
    let instrument = settings.instrument();
    let record = progress.get(&instrument.key()).cloned().unwrap_or_default();
    let title = format!("{} Exercises", settings.instrument.name());
    let tuning = match TuningPreset::matching(&settings.tuning) {
        Some(TuningPreset::Standard) => "standard tuning".to_string(),
        Some(preset) => format!("{} tuning", preset.name().to_lowercase()),
        None => "a custom tuning".to_string(),
    };
    let caption = format!("{}, {} strings in {tuning}, {} frets", settings.instrument.name(), settings.strings, settings.frets);
    commands.spawn(screen_root(Screen::Menu)).with_children(|root| {
        spawn_top_bar(root, &title, Screen::Menu, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            list.spawn(Node {
                width: percent(100),
                max_width: px(COLUMN),
                flex_direction: FlexDirection::Column,
                row_gap: px(4),
                ..default()
            })
            .with_children(|column| {
                spawn_strings(column, &instrument, fonts);
                column.spawn((
                    Node { margin: UiRect::bottom(px(14)), ..default() },
                    children![label(caption, fonts.text(size::TEXT), colors::dim())],
                ));
                for (name, description, screen) in ENTRIES {
                    let status = status_of(screen, &settings, &record);
                    spawn_entry(column, name, description, status, screen, fonts);
                }
            });
        });
    });
}

/// The open strings, the highest on top as on the fretboard and in the tablature: their names,
/// and lines as thick as the strings.
fn spawn_strings(column: &mut ChildSpawnerCommands, instrument: &crate::music::Instrument, fonts: &Fonts) {
    let strings = instrument.strings();
    column.spawn(Node { flex_direction: FlexDirection::Column, padding: UiRect::vertical(px(6)), ..default() }).with_children(|s| {
        for string in (0..strings).rev() {
            let thickness = 1.0 + (strings - 1 - string) as f32 * 0.45;
            s.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, height: px(16), ..default() })
                .with_children(|row| {
                    row.spawn((
                        Node { width: px(28), ..default() },
                        children![label(instrument.string_label(string), fonts.display(size::TEXT), colors::accent())],
                    ));
                    row.spawn((
                        Node { flex_grow: 1.0, height: px(thickness), ..default() },
                        BackgroundColor(crate::ui::colors::STRING.with_alpha(0.75)),
                    ));
                });
        }
    });
}

/// Where the player is in an entry, a few words.
fn status_of(screen: Screen, settings: &Settings, record: &InstrumentProgress) -> String {
    let catalog = settings.catalog();
    let stars = |page: Page| -> u32 { catalog.on_page(page).filter_map(|e| record.exercises.get(&e.key)).map(|r| r.stars() as u32).sum() };
    let possible = |page: Page| catalog.on_page(page).count() * 3;
    match screen {
        Screen::Drills => format!("{} of {} stars", stars(Page::Drills), possible(Page::Drills)),
        Screen::Scales => format!("{} of {} stars", stars(Page::Scales), possible(Page::Scales)),
        Screen::Learn => {
            let total = learning_order(&settings.instrument(), settings.accidentals).len();
            format!("{} of {total} notes", record.learned.min(total))
        }
        Screen::Find => {
            let found = record.finding.values().filter(|n| n.correct > 0).count();
            if found == 0 { String::new() } else { format!("{found} notes found") }
        }
        _ => String::new(),
    }
}

fn spawn_entry(column: &mut ChildSpawnerCommands, name: &str, description: &str, status: String, screen: Screen, fonts: &Fonts) {
    column
        .spawn(button_frame(colors::bg()))
        .insert((
            Node {
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                column_gap: px(16),
                padding: UiRect::axes(px(10), px(10)),
                border: UiRect::top(px(1)),
                border_radius: BorderRadius::all(px(6)),
                ..default()
            },
            BorderColor::all(colors::button()),
        ))
        .with_children(|row| {
            row.spawn((Node { flex_direction: FlexDirection::Column, flex_shrink: 1.0, ..default() }, Pickable::IGNORE)).with_children(
                |text| {
                    text.spawn(label(name, fonts.display(size::TITLE), colors::text()));
                    text.spawn(label(description, fonts.text(size::TEXT), colors::dim()));
                },
            );
            row.spawn(label(status, fonts.display(size::HEADING), colors::accent()));
        })
        .observe(move |_: On<PointerClick>, mut next: ResMut<NextState<Screen>>| {
            next.set(screen);
        });
}
