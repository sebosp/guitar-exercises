//! The main menu: the games, the statistics and the settings.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::settings::Settings;
use crate::ui::{Fonts, Screen, button_frame, colors, label, screen_root, spawn_top_bar};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Menu), spawn);
    }
}

const ENTRIES: [(&str, &str, Screen); 5] = [
    ("Finger drills", "The four fingers in every order, wider spans, string crossings", Screen::Drills),
    ("Learn notes", "Read a note on the staff, see it on the fretboard, play it", Screen::Learn),
    ("Find them all", "A note on the staff: tap every place it is on the fretboard", Screen::Find),
    ("Statistics", "Accuracy and speed of the drills and the notes", Screen::Stats),
    ("Settings", "Instrument, strings, tuning, microphone, hints", Screen::Settings),
];

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>) {
    let instrument = settings.instrument();
    let strings: Vec<String> = (0..instrument.strings()).map(|s| instrument.string_label(s)).collect();
    let subtitle = format!(
        "{} · {} strings · {} ({}) · {} frets",
        settings.instrument.name(),
        settings.strings,
        settings.tuning.name(),
        strings.join(" "),
        settings.frets
    );
    let title = format!("{} Exercises", settings.instrument.name());
    commands.spawn(screen_root(Screen::Menu)).with_children(|root| {
        spawn_top_bar(root, &title, Screen::Menu, &fonts);
        root.spawn((
            Node {
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: px(12),
                padding: UiRect::all(px(16)),
                ..default()
            },
            crate::ui::Scrollable,
            ScrollPosition::default(),
        ))
        .observe(crate::ui::scroll_on_wheel)
        .with_children(|column| {
            column.spawn(label(subtitle, fonts.text(17.0), colors::DIM));
            for (name, description, screen) in ENTRIES {
                column
                    .spawn(button_frame(colors::BUTTON))
                    .insert(Node {
                        width: px(440),
                        max_width: percent(100),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Start,
                        padding: UiRect::axes(px(18), px(10)),
                        border_radius: BorderRadius::all(px(10)),
                        ..default()
                    })
                    .with_children(|b| {
                        b.spawn(label(name, fonts.bold(21.0), colors::TEXT));
                        b.spawn(label(description, fonts.text(15.0), colors::DIM));
                    })
                    .observe(move |_: On<PointerClick>, mut next: ResMut<NextState<Screen>>| {
                        next.set(screen);
                    });
            }
        });
    });
}
