//! The help, shown on the first start and from the menu or F1: how to play, what the stars, the
//! tempo and the tiers mean, the streak, the keys.
use bevy::prelude::*;

use crate::settings::Settings;
use crate::ui::{Fonts, Screen, colors, label, screen_root, scroll_area, size, spawn_top_bar};

pub struct HelpPlugin;

impl Plugin for HelpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Help), spawn).add_systems(OnEnter(Screen::Menu), first_start).add_systems(Update, f1);
    }
}

/// The help once, before the menu of the first start.
fn first_start(mut settings: ResMut<Settings>, mut next: ResMut<NextState<Screen>>) {
    if !settings.help_seen {
        settings.help_seen = true;
        settings.save();
        next.set(Screen::Help);
    }
}

fn f1(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<Screen>>) {
    if keys.just_pressed(KeyCode::F1) {
        next.set(Screen::Help);
    }
}

const FULL_SCREEN: &str = if cfg!(target_os = "macos") { "F11 or Cmd+Ctrl+F" } else { "F11" };
const ZOOM: &str = if cfg!(target_os = "macos") {
    "Cmd and the mouse wheel, Cmd+= Cmd+- Cmd+0, a pinch"
} else {
    "Ctrl and the mouse wheel, Ctrl+= Ctrl+- Ctrl+0, a pinch"
};

fn spawn(mut commands: Commands, fonts: Res<Fonts>, settings: Res<Settings>) {
    let fonts = &*fonts;
    let goal = settings.daily_goal_minutes;
    let sections: [(&str, String); 7] = [
        (
            "Playing",
            "Play the note on the instrument, or tap it on the fretboard. The microphone listens when it is on in the settings; a pickup or an audio interface hears best. A pause shows where the next note is and with which finger.".to_string(),
        ),
        (
            "Stars",
            "One star for 80% of the notes right, two for 90%, three for all of them. A star opens the next level, three levels of a row open the next row, 30 stars reveal the extra exercises.".to_string(),
        ),
        (
            "Tempo",
            "With the metronome a round is clean at 90% right and 80% in time. A clean round speeds up 5 BPM, up to the target; a round under that keeps the tempo, a second one in a row slows down 5 BPM. Set the tempo and the target in the drill.".to_string(),
        ),
        (
            "Tiers",
            "The pips of a level are its fastest clean round, in notes a minute: Basic for a clean round, Medium from 160, Advanced from 240, Legendary from 360.".to_string(),
        ),
        (
            "Songs",
            "Drop a Guitar Pro file on the window, or open one, then play a section of it at a time. The speed starts at 60% of the score's tempo and goes up 5% after each clean round, up to the target: set it to 110% or 120% to play it faster than written.".to_string(),
        ),
        (
            "Streak",
            format!("The minutes played count for the day; {goal} minutes a day keep the streak going. Change the goal in the settings."),
        ),
        (
            "Keys",
            format!("Esc goes back. {FULL_SCREEN}: full screen. Zoom: {ZOOM}. F1: this help."),
        ),
    ];
    commands.spawn(screen_root(Screen::Help)).with_children(|root| {
        spawn_top_bar(root, "Help", Screen::Help, fonts);
        root.spawn(scroll_area()).observe(crate::ui::scroll_on_wheel).observe(crate::ui::scroll_on_drag).with_children(|list| {
            list.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(18), max_width: px(680), ..default() }).with_children(
                |column| {
                    for (title, text) in sections {
                        column.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4), ..default() }).with_children(|s| {
                            s.spawn(label(title, fonts.display(size::TITLE), colors::accent()));
                            s.spawn(label(text, fonts.text(size::LEAD), colors::text()));
                        });
                    }
                },
            );
        });
    });
}
