//! A staff of five lines with its clef and one note, the ledger lines it needs, and as a hint the
//! letters of the lines or the spaces (the mnemonics).
//!
//! The music font is Noto Music: its em is the height of the staff and the baseline of the clefs
//! is the bottom line.
use bevy::prelude::*;

use crate::music::{Clef, LETTERS, Spelled, staff_position};
use crate::ui::{Fonts, colors, label};

/// Half the distance between two lines: one step of the staff.
const HALF: f32 = 8.0;
const LINE_GAP: f32 = 2.0 * HALF;
/// The highest and the lowest positions drawn, 0 is the bottom line.
const TOP: i32 = 16;
const BOTTOM: i32 = -10;
const PAD: f32 = 6.0;
const WIDTH: f32 = 300.0;
const LINES_LEFT: f32 = 12.0;
/// The lines stop there, the mnemonic letters are right of them.
const LINES_RIGHT: f32 = 232.0;
const NOTE_X: f32 = 150.0;
/// The music font size, the em is the staff height.
const MUSIC_SIZE: f32 = 4.0 * LINE_GAP;
/// The baseline of a text line from its top, in ems, measured on the screen.
const BASELINE: f32 = 1.10;

#[derive(Component)]
pub struct StaffView {
    pub clef: Clef,
}

/// Which letters to write next to the staff, as a hint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Letters {
    #[default]
    None,
    Lines,
    Spaces,
    /// Lines and spaces, the note's letter highlighted.
    All,
}

#[derive(Debug, Clone, Default)]
pub struct StaffDrawing {
    /// The written note.
    pub note: Option<Spelled>,
    pub color: Option<Color>,
    pub letters: Letters,
}

fn y_of(position: i32) -> f32 {
    PAD + (TOP - position) as f32 * HALF
}

pub fn spawn_staff(parent: &mut ChildSpawnerCommands, clef: Clef) {
    parent.spawn((
        Node { width: px(WIDTH), height: px(y_of(BOTTOM) + PAD), border_radius: BorderRadius::all(px(8)), flex_shrink: 0.0, ..default() },
        BackgroundColor(colors::PAPER),
        StaffView { clef },
        Pickable::IGNORE,
    ));
}

fn line(left: f32, top: f32, width: f32, height: f32, color: Color) -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            left: px(left),
            top: px(top - height / 2.0),
            width: px(width),
            height: px(height),
            ..default()
        },
        BackgroundColor(color),
        Pickable::IGNORE,
    )
}

/// A text placed with its baseline at `baseline`.
fn glyph(text: &str, left: f32, baseline: f32, size: f32, font: TextFont, color: Color) -> impl Bundle {
    (
        Node { position_type: PositionType::Absolute, left: px(left), top: px(baseline - BASELINE * size), ..default() },
        Text::new(text),
        font,
        TextColor(color),
        TextLayout::no_wrap(),
        Pickable::IGNORE,
    )
}

/// Redraws a staff.
pub fn draw_staff(commands: &mut Commands, staff: (Entity, &StaffView), drawing: &StaffDrawing, fonts: &Fonts) {
    let (entity, view) = staff;
    let clef = view.clef;
    commands.entity(entity).despawn_children();
    commands.entity(entity).with_children(|s| {
        for l in 0..5 {
            s.spawn(line(LINES_LEFT, y_of(2 * l), LINES_RIGHT - LINES_LEFT, 1.5, colors::INK));
        }
        // The clefs sit with their baseline on the bottom line.
        s.spawn(glyph(clef.glyph(), LINES_LEFT + 4.0, y_of(0), MUSIC_SIZE, fonts.music(MUSIC_SIZE), colors::INK));
        if drawing.letters != Letters::None {
            spawn_letters(s, clef, drawing, fonts);
        }
        let Some(note) = drawing.note else {
            return;
        };
        let position = staff_position(clef, note);
        let color = drawing.color.unwrap_or(colors::INK);
        // The ledger lines, below or above the staff.
        let ledgers: Vec<i32> = if position < 0 {
            (position..0).filter(|p| p % 2 == 0).collect()
        } else if position > 8 {
            (9..=position).filter(|p| p % 2 == 0).collect()
        } else {
            vec![]
        };
        for p in ledgers {
            s.spawn(line(NOTE_X - 8.0, y_of(p), 38.0, 1.5, colors::INK));
        }
        // A whole note: a tilted ring.
        let (w, h) = (22.0, 15.0);
        s.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(NOTE_X),
                top: px(y_of(position) - h / 2.0),
                width: px(w),
                height: px(h),
                border: UiRect::axes(px(4.5), px(2.5)),
                border_radius: BorderRadius::MAX,
                ..default()
            },
            BorderColor::all(color),
            UiTransform { rotation: Rot2::degrees(-25.0), ..default() },
            Pickable::IGNORE,
        ));
        if let Some(accidental) = note.accidental_glyph() {
            s.spawn(glyph(accidental, NOTE_X - 22.0, y_of(position), MUSIC_SIZE, fonts.music(MUSIC_SIZE), color));
        }
    });
}

/// The letters of the lines and the spaces right of the staff, the note's letter in color.
fn spawn_letters(s: &mut ChildSpawnerCommands, clef: Clef, drawing: &StaffDrawing, fonts: &Fonts) {
    let bottom_letter = (clef.bottom_line() % 7) as usize;
    let note_position = drawing.note.map(|n| staff_position(clef, n));
    for p in -1..=9 {
        let on_line = p % 2 == 0;
        let shown = match drawing.letters {
            Letters::Lines => on_line && (0..=8).contains(&p),
            Letters::Spaces => !on_line && (1..=7).contains(&p),
            Letters::All => true,
            Letters::None => false,
        };
        if !shown {
            continue;
        }
        let letter = LETTERS[(bottom_letter as i32 + p).rem_euclid(7) as usize];
        let color = if note_position == Some(p) { colors::BAD } else { colors::REVEAL };
        // The lines in one column, the spaces in the next, with a tick to their line or space.
        let x = if on_line { LINES_RIGHT + 8.0 } else { LINES_RIGHT + 30.0 };
        s.spawn((
            Node { position_type: PositionType::Absolute, left: px(x), top: px(y_of(p) - 8.0), ..default() },
            label(letter.to_string(), fonts.bold(13.0), color),
        ));
        if on_line {
            s.spawn(line(LINES_RIGHT, y_of(p), 6.0, 1.0, color));
        } else {
            s.spawn(line(LINES_RIGHT + 2.0, y_of(p), 26.0, 1.0, color.with_alpha(0.5)));
        }
    }
}
