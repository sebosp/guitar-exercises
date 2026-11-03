//! The shapes, the first level of "Find them all": a page per shape (the same note on the next
//! string, the octaves), the notes linked by lines on the fretboard and played out loud. The
//! notes to find pulse; tapping one plays it. The first time, every note of a page is tapped
//! before the next one; the last page leads to "every octave".
use bevy::picking::events::PointerClick;
use bevy::prelude::*;

use crate::fretboard::{FretTapped, LinksLayer, Mark, MarksLayer, set_links, set_marks, spawn_fretboard};
use crate::metronome::PlayNote;
use crate::music::{Instrument, Position, Spelled};
use crate::screens::find::FindLevel;
use crate::settings::Settings;
use crate::shapes::{Page, pages};
use crate::ui::{Fonts, Screen, button, colors, label, screen_root, scroll_area, size, spawn_top_bar};

/// The seconds between the notes of a shape played.
const NOTE_GAP: f32 = 0.55;

pub struct FindTutorialPlugin;

impl Plugin for FindTutorialPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::FindTutorial), spawn)
            .add_systems(Update, (tap, render).chain().run_if(in_state(Screen::FindTutorial)));
    }
}

#[derive(Resource)]
struct Tutorial {
    instrument: Instrument,
    pages: Vec<Page>,
    page: usize,
    tapped: Vec<Position>,
    /// Every target must be tapped before the next page: the first time through.
    guided: bool,
    dirty: bool,
}

impl Tutorial {
    fn current(&self) -> &Page {
        &self.pages[self.page]
    }

    fn complete(&self) -> bool {
        self.current().targets.iter().all(|t| self.tapped.contains(t))
    }

    /// The notes of the page, in order: the ones shown, then the ones to find.
    fn sequence(&self) -> Vec<i32> {
        let page = self.current();
        page.shown.iter().chain(&page.targets).map(|p| self.instrument.midi_at(*p)).collect()
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum TutorialText {
    Step,
    Title,
    Text,
    Mnemonic,
    Task,
}

#[derive(Component)]
struct NextButton;

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>, mut play: MessageWriter<PlayNote>) {
    let fonts = &*fonts;
    let instrument = settings.instrument();
    let tutorial = Tutorial {
        instrument: instrument.clone(),
        pages: pages(&instrument),
        page: 0,
        tapped: vec![],
        guided: !settings.find_tutorial_done,
        dirty: true,
    };
    for (i, midi) in tutorial.sequence().into_iter().enumerate() {
        play.write(PlayNote { midi, delay: 0.3 + i as f32 * NOTE_GAP });
    }
    commands.insert_resource(tutorial);
    commands.spawn(screen_root(Screen::FindTutorial)).with_children(|root| {
        spawn_top_bar(root, "Find them all: the shapes", Screen::FindTutorial, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|body| {
            body.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(6), max_width: px(760), ..default() }).with_children(
                |text| {
                    text.spawn((label("", fonts.text(size::TEXT), colors::dim()), TutorialText::Step));
                    text.spawn((label("", fonts.display(size::TITLE), colors::text()), TutorialText::Title));
                    text.spawn((label("", fonts.text(size::LEAD), colors::text()), TutorialText::Text));
                    text.spawn((label("", fonts.display(size::HEADING), colors::accent()), TutorialText::Mnemonic));
                    text.spawn((label("", fonts.text(size::TEXT), colors::reveal()), TutorialText::Task));
                },
            );
            spawn_fretboard(body, &instrument, settings.left_handed, fonts);
            body.spawn(Node {
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                column_gap: px(10),
                row_gap: px(8),
                ..default()
            })
            .with_children(|row| {
                row.spawn(button("Previous", fonts, colors::button())).observe(
                    |_: On<PointerClick>, mut t: ResMut<Tutorial>, mut play: MessageWriter<PlayNote>| {
                        if t.page > 0 {
                            t.page -= 1;
                            turn(&mut t, &mut play);
                        }
                    },
                );
                row.spawn(button("Hear them", fonts, colors::button())).observe(
                    |_: On<PointerClick>, t: Res<Tutorial>, mut play: MessageWriter<PlayNote>| {
                        for (i, midi) in t.sequence().into_iter().enumerate() {
                            play.write(PlayNote { midi, delay: i as f32 * NOTE_GAP });
                        }
                    },
                );
                row.spawn((button("Next", fonts, colors::button()), NextButton)).observe(
                    |_: On<PointerClick>,
                     mut t: ResMut<Tutorial>,
                     mut play: MessageWriter<PlayNote>,
                     mut settings: ResMut<Settings>,
                     mut level: ResMut<FindLevel>,
                     mut next: ResMut<NextState<Screen>>| {
                        if t.guided && !t.complete() {
                            return;
                        }
                        if t.page + 1 < t.pages.len() {
                            t.page += 1;
                            turn(&mut t, &mut play);
                        } else {
                            // Done: on to every octave.
                            settings.find_tutorial_done = true;
                            settings.save();
                            *level = FindLevel::EveryOctave;
                            next.set(Screen::Find);
                        }
                    },
                );
            });
        });
    });
}

/// Another page: nothing tapped, its notes played.
fn turn(t: &mut Tutorial, play: &mut MessageWriter<PlayNote>) {
    t.tapped.clear();
    t.dirty = true;
    for (i, midi) in t.sequence().into_iter().enumerate() {
        play.write(PlayNote { midi, delay: i as f32 * NOTE_GAP });
    }
}

fn tap(mut t: ResMut<Tutorial>, mut taps: MessageReader<FretTapped>, mut play: MessageWriter<PlayNote>) {
    for FretTapped(position) in taps.read() {
        let page = t.current();
        let known = page.targets.contains(position) || page.shown.contains(position);
        if known {
            play.write(PlayNote { midi: t.instrument.midi_at(*position), delay: 0.0 });
        }
        if page.targets.contains(position) && !t.tapped.contains(position) {
            t.tapped.push(*position);
            t.dirty = true;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn render(
    mut t: ResMut<Tutorial>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    marks: Query<(Entity, &MarksLayer)>,
    links: Query<Entity, With<LinksLayer>>,
    mut texts: Query<(&TutorialText, &mut Text)>,
    mut nexts: Query<(&mut crate::ui::Btn, &mut BackgroundColor), With<NextButton>>,
    mut expected: ResMut<crate::screens::Expected>,
) {
    if !t.dirty {
        return;
    }
    t.dirty = false;
    let page = t.current().clone();
    let next_target = page.targets.iter().copied().find(|p| !t.tapped.contains(p));
    *expected = crate::screens::Expected { midi: next_target.map(|p| t.instrument.midi_at(p)), position: next_target };
    let name = |p: &Position| Spelled::from_midi(t.instrument.midi_at(*p), false).name();
    if let Ok(layer) = marks.single() {
        let mut list: Vec<Mark> = page.shown.iter().map(|p| Mark::new(*p, name(p), colors::accent()).big()).collect();
        for target in &page.targets {
            list.push(if t.tapped.contains(target) {
                Mark::new(*target, name(target), colors::ok())
            } else {
                Mark::new(*target, "?", colors::reveal()).ring(Color::WHITE).pulse()
            });
        }
        set_marks(&mut commands, layer, &list, &fonts);
    }
    if let Ok(layer) = links.single() {
        set_links(&mut commands, layer, &page.links, colors::accent().with_alpha(0.7));
    }
    let left = page.targets.iter().filter(|p| !t.tapped.contains(p)).count();
    let last = t.page + 1 == t.pages.len();
    let task = if left > 0 {
        format!("Tap the {left} {} pulsing on the fretboard.", if left == 1 { "note" } else { "notes" })
    } else if last {
        "Every one found. Next: every octave of a note, on your own.".to_string()
    } else {
        "Every one found.".to_string()
    };
    let step = format!("{} of {}", t.page + 1, t.pages.len());
    for (kind, mut text) in &mut texts {
        text.0 = match kind {
            TutorialText::Step => step.clone(),
            TutorialText::Title => page.title.clone(),
            TutorialText::Text => page.text.clone(),
            TutorialText::Mnemonic => page.mnemonic.clone(),
            TutorialText::Task => task.clone(),
        };
    }
    let ready = !t.guided || left == 0;
    for (mut btn, mut bg) in &mut nexts {
        btn.color = if ready { colors::selected() } else { colors::locked() };
        btn.hover = if ready { colors::button_hover() } else { btn.color };
        bg.0 = btn.color;
    }
}
