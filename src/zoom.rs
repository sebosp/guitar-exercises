//! The zoom of the whole window, and F11 for the full screen. The window is scaled to fit its
//! size first: a big screen, or the full screen, shows everything bigger, a phone not under 75%
//! (smaller text is too small to read; the screens scroll and wrap). Over that, Ctrl (Cmd on
//! macOS) and the mouse wheel, a pinch of the trackpad or of two fingers on a touch screen,
//! Ctrl+= Ctrl+- Ctrl+0, or the buttons of the settings zoom in and out, to read the screen from
//! the instrument a few steps away. The zoom is saved with the settings.
//!
//! The full screen: F11, or Cmd+Ctrl+F on macOS. In the browser, F11, Ctrl with the wheel and
//! the keys are the browser's own (its full screen and its zoom), the pinch and the buttons zoom
//! the game.
use bevy::input::ButtonState;
use bevy::input::gestures::PinchGesture;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::{MonitorSelection, PrimaryWindow, WindowMode};

use crate::settings::Settings;

pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 3.0;
/// The smallest scale that fits a window: under it the text is too small on a phone.
const MIN_FIT: f32 = 0.75;
/// The zoom of a key press or a button, a factor.
const STEP: f32 = 1.1;
/// The size of the window at 100%: the screens are laid out for it.
const DESIGN: Vec2 = Vec2::new(1150.0, 800.0);

/// The scale that fits the window.
pub fn fit(window: Vec2) -> f32 {
    if window.x <= 0.0 || window.y <= 0.0 {
        return 1.0;
    }
    (window / DESIGN).min_element().clamp(MIN_FIT, MAX_ZOOM)
}

/// Changes the zoom.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub enum Zoom {
    /// Multiplies it.
    By(f32),
    Reset,
}

impl Zoom {
    pub const IN: Zoom = Zoom::By(STEP);
    pub const OUT: Zoom = Zoom::By(1.0 / STEP);

    /// The zoom of a key pressed with Ctrl, as in the browsers.
    pub fn of_key(key: &Key) -> Option<Zoom> {
        let Key::Character(c) = key else {
            return None;
        };
        match c.as_str() {
            "=" | "+" => Some(Zoom::IN),
            "-" | "_" => Some(Zoom::OUT),
            "0" => Some(Zoom::Reset),
            _ => None,
        }
    }

    /// The new zoom factor over the base scale, the zoom stays between the bounds.
    fn apply(self, factor: f32, base: f32) -> f32 {
        match self {
            Zoom::By(by) => (factor * by).clamp(MIN_ZOOM / base, MAX_ZOOM / base),
            Zoom::Reset => 1.0,
        }
    }
}

pub struct ZoomPlugin;

impl Plugin for ZoomPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Zoom>().add_systems(Update, ((gestures, two_fingers, apply_zoom).chain(), full_screen));
    }
}

/// Ctrl is held, Cmd on macOS: the wheel zooms instead of scrolling.
pub fn ctrl_held(keys: &ButtonInput<KeyCode>) -> bool {
    keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight])
        || (cfg!(target_os = "macos") && keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight]))
}

/// Ctrl and the wheel or a key, or a pinch.
fn gestures(
    mut wheel: MessageReader<MouseWheel>,
    mut pinch: MessageReader<PinchGesture>,
    mut typed: MessageReader<KeyboardInput>,
    keys: Res<ButtonInput<KeyCode>>,
    mut zoom: MessageWriter<Zoom>,
) {
    let ctrl = ctrl_held(&keys) && !cfg!(target_arch = "wasm32");
    for event in wheel.read() {
        if !ctrl {
            continue;
        }
        let notches = match event.unit {
            MouseScrollUnit::Line => event.y,
            MouseScrollUnit::Pixel => event.y / 40.0,
        };
        if notches != 0.0 {
            zoom.write(Zoom::By(STEP.powf(notches.clamp(-3.0, 3.0))));
        }
    }
    for PinchGesture(delta) in pinch.read() {
        zoom.write(Zoom::By((1.0 + delta).clamp(0.5, 2.0)));
    }
    for key in typed.read() {
        if ctrl
            && key.state == ButtonState::Pressed
            && let Some(change) = Zoom::of_key(&key.logical_key)
        {
            zoom.write(change);
        }
    }
}

/// Two fingers on a touch screen moving apart or together: the distance between them, from the
/// last frame.
fn two_fingers(touches: Res<Touches>, mut last: Local<Option<f32>>, mut zoom: MessageWriter<Zoom>) {
    let fingers: Vec<Vec2> = touches.iter().take(3).map(|t| t.position()).collect();
    let [a, b] = fingers[..] else {
        *last = None;
        return;
    };
    let distance = a.distance(b);
    if let Some(before) = *last
        && before > 10.0
        && (distance - before).abs() > 0.5
    {
        zoom.write(Zoom::By((distance / before).clamp(0.8, 1.25)));
    }
    *last = Some(distance);
}

fn apply_zoom(
    mut changes: MessageReader<Zoom>,
    mut settings: ResMut<Settings>,
    mut scale: ResMut<UiScale>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let base = fit(window.size());
    let mut factor = settings.zoom;
    for change in changes.read() {
        factor = change.apply(factor, base);
    }
    if factor != settings.zoom {
        settings.zoom = factor;
        settings.save();
    }
    let zoom = (base * factor).clamp(MIN_ZOOM, MAX_ZOOM);
    if scale.0 != zoom {
        scale.0 = zoom;
    }
}

/// F11, or Cmd+Ctrl+F on macOS: the full screen, and back. Not in the browser: its F11 does it.
fn full_screen(keys: Res<ButtonInput<KeyCode>>, mut window: Single<&mut Window, With<PrimaryWindow>>) {
    let mac = cfg!(target_os = "macos")
        && keys.just_pressed(KeyCode::KeyF)
        && keys.any_pressed([KeyCode::SuperLeft, KeyCode::SuperRight])
        && keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if !cfg!(target_arch = "wasm32") && (keys.just_pressed(KeyCode::F11) || mac) {
        window.mode = match window.mode {
            WindowMode::Windowed => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
            _ => WindowMode::Windowed,
        };
    }
}

/// The zoom in percent, for the settings.
pub fn percent(zoom: f32) -> String {
    format!("{:.0}%", zoom * 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps_stay_in_range() {
        let base = 2.0;
        let mut factor = 1.0;
        for _ in 0..50 {
            factor = Zoom::IN.apply(factor, base);
        }
        assert_eq!(factor * base, MAX_ZOOM);
        assert_eq!(Zoom::Reset.apply(factor, base), 1.0);
        assert!((Zoom::OUT.apply(Zoom::IN.apply(1.0, base), base) - 1.0).abs() < 1e-6);
        // A full HD screen shows everything bigger, a phone sideways not under 75%.
        assert_eq!(fit(Vec2::new(2300.0, 1600.0)), 2.0);
        assert!((fit(Vec2::new(1920.0, 1080.0)) - 1.35).abs() < 1e-6);
        assert_eq!(fit(Vec2::new(800.0, 360.0)), MIN_FIT);
        assert_eq!(Zoom::of_key(&Key::Character("=".into())), Some(Zoom::IN));
        assert_eq!(Zoom::of_key(&Key::Character("a".into())), None);
    }
}
