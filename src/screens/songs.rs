//! The songs page: drop a Guitar Pro file or open one, then each song with its tracks and its
//! sections, each section a level with its stars and its speed, from 60% of the tempo of the
//! score up to the target.
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::library::{HAS_DIALOG, Library, OpenDialog, section_key};
use crate::progress::{InstrumentProgress, Progress};
use crate::screens::drills::SelectedLevel;
use crate::settings::Settings;
use crate::tempo::{Tier, tempo_of};
use crate::ui::{
    Fonts, Icons, ReturnTo, Screen, button, button_frame, colors, label, screen_root, scroll_area, size, spawn_stars, spawn_top_bar,
};

pub struct SongsPlugin;

impl Plugin for SongsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Songs), spawn);
    }
}

const TILE: f32 = 132.0;

fn spawn(
    mut commands: Commands,
    fonts: Res<Fonts>,
    icons: Res<Icons>,
    settings: Res<Settings>,
    progress: Res<Progress>,
    library: Res<Library>,
) {
    let fonts = &*fonts;
    let icons = &*icons;
    let record = progress.get(&settings.instrument().key()).cloned().unwrap_or_default();
    commands.spawn(screen_root(Screen::Songs)).with_children(|root| {
        spawn_top_bar(root, "Songs", Screen::Songs, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            spawn_import(list, &library, fonts);
            for song in &library.songs {
                spawn_song(list, song, &record, fonts, icons);
            }
        });
    });
}

/// Where the files come in.
fn spawn_import(list: &mut ChildSpawnerCommands, library: &Library, fonts: &Fonts) {
    list.spawn((
        Node {
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            align_items: AlignItems::Center,
            column_gap: px(16),
            row_gap: px(8),
            padding: UiRect::all(px(16)),
            border: UiRect::all(px(2)),
            border_radius: BorderRadius::all(px(10)),
            ..default()
        },
        BorderColor::all(colors::button_hover()),
    ))
    .with_children(|zone| {
        let how = if cfg!(target_os = "android") {
            let folder = folder_text();
            format!("Copy Guitar Pro files to {folder}, they show here.")
        } else if library.songs.is_empty() {
            "Drop a Guitar Pro file on the window, .gp .gpx .gp5 .gp4 or .gp3.".to_string()
        } else {
            "Drop another Guitar Pro file on the window.".to_string()
        };
        zone.spawn(label(how, fonts.text(size::LEAD), colors::text()));
        if HAS_DIALOG {
            zone.spawn(button("Open a file", fonts, colors::selected()))
                .observe(|_: On<PointerClick>, mut open: ResMut<OpenDialog>| open.0 = true);
        }
    });
    if !library.notice.is_empty() {
        list.spawn(label(&library.notice, fonts.bold(size::TEXT), colors::accent()));
    }
}

fn folder_text() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        crate::storage::songs_folder().map(|p| p.display().to_string()).unwrap_or_else(|| "the songs folder".to_string())
    }
    #[cfg(target_arch = "wasm32")]
    {
        "the songs folder".to_string()
    }
}

fn spawn_song(
    list: &mut ChildSpawnerCommands,
    song: &crate::library::LibrarySong,
    record: &InstrumentProgress,
    fonts: &Fonts,
    icons: &Icons,
) {
    let data = &song.data;
    let id = song.saved.id.clone();
    let track = song.saved.track.min(data.tracks.len().saturating_sub(1));
    list.spawn(Node { height: px(8), ..default() });
    list.spawn(Node {
        flex_direction: FlexDirection::Row,
        flex_wrap: FlexWrap::Wrap,
        align_items: AlignItems::Baseline,
        column_gap: px(12),
        ..default()
    })
    .with_children(|row| {
        row.spawn(label(&data.title, fonts.display(size::TITLE), colors::text()));
        if !data.artist.is_empty() {
            row.spawn(label(&data.artist, fonts.text(size::LEAD), colors::dim()));
        }
        let remove = id.clone();
        row.spawn(button("Remove", fonts, colors::button())).observe(
            move |_: On<PointerClick>, mut library: ResMut<Library>, mut next: ResMut<NextState<Screen>>| {
                library.remove(&remove);
                library.notice.clear();
                next.set(Screen::Songs);
            },
        );
    });
    if data.tracks.len() > 1 {
        list.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: px(8), row_gap: px(6), ..default() })
            .with_children(|row| {
                for (i, t) in data.tracks.iter().enumerate() {
                    let strings: Vec<String> =
                        (0..t.open.len()).rev().map(|s| crate::music::Spelled::from_midi(t.open[s], false).name()).collect();
                    let name = if t.name.is_empty() { format!("Track {}", i + 1) } else { t.name.clone() };
                    let color = if i == track { colors::selected() } else { colors::button() };
                    let id = id.clone();
                    row.spawn(button(format!("{name} ({})", strings.join(" ")), fonts, color)).observe(
                        move |_: On<PointerClick>, mut library: ResMut<Library>, mut next: ResMut<NextState<Screen>>| {
                            library.set_track(&id, i);
                            next.set(Screen::Songs);
                        },
                    );
                }
            });
    }
    let tempo = data.measures.first().map_or(120.0, |m| m.tempo);
    list.spawn(label(format!("{} measures at {tempo:.0} BPM", data.measures.len()), fonts.text(size::TEXT), colors::dim()));
    list.spawn(Node { flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::Wrap, column_gap: px(6), row_gap: px(6), ..default() })
        .with_children(|grid| {
            for section in data.sections() {
                let key = section_key(&id, track, &section.measures);
                let drill = record.exercises.get(&key);
                let tier = drill.and_then(Tier::of);
                grid.spawn(button_frame(colors::button()))
                    .insert(Node {
                        width: px(TILE),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        row_gap: px(2),
                        padding: UiRect::axes(px(4), px(4)),
                        border_radius: BorderRadius::all(px(6)),
                        ..default()
                    })
                    .with_children(|tile| {
                        tile.spawn(label(&section.name, fonts.display(19.0), colors::text()));
                        if !section.name.starts_with("Bars ") {
                            tile.spawn(label(
                                format!("bars {}-{}", section.measures.start + 1, section.measures.end),
                                fonts.text(12.0),
                                colors::dim(),
                            ));
                        }
                        spawn_stars(tile, drill.map_or(0, |d| d.stars()), 16.0, icons);
                        let detail = match drill {
                            Some(d) if d.bpm > 0 => {
                                let (speed, target) = tempo_of(d);
                                if d.target_reached { format!("{target}% done") } else { format!("{speed}% of {target}%") }
                            }
                            Some(d) if d.plays > 0 => format!("best {:.0}%", d.best_accuracy * 100.0),
                            _ => "new".to_string(),
                        };
                        tile.spawn(label(detail, fonts.text(12.0), colors::dim()));
                        if let Some(tier) = tier {
                            tile.spawn((
                                Node {
                                    flex_direction: FlexDirection::Row,
                                    align_items: AlignItems::Center,
                                    column_gap: px(5),
                                    ..default()
                                },
                                Pickable::IGNORE,
                            ))
                            .with_children(|line| {
                                crate::screens::drills::spawn_tier_pips(line, tier);
                                line.spawn(label(tier.name(), fonts.bold(12.0), colors::text()));
                            });
                        }
                    })
                    .observe(
                        move |_: On<PointerClick>,
                              mut selected: ResMut<SelectedLevel>,
                              mut back: ResMut<ReturnTo>,
                              mut next: ResMut<NextState<Screen>>| {
                            selected.0 = key.clone();
                            back.0 = Screen::Songs;
                            next.set(Screen::Drill);
                        },
                    );
            }
        });
}
