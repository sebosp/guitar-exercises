use bevy::prelude::*;
use guitar_exercises::GuitarExercisesPlugin;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "Guitar Exercises".into(),
            // The app id of Wayland and the class of X11.
            name: Some("guitar-exercises".into()),
            resolution: (1280, 860).into(),
            // The web build fills the page, see index.html.
            canvas: Some("#guitar-exercises".into()),
            fit_canvas_to_parent: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(GuitarExercisesPlugin);
    #[cfg(not(target_arch = "wasm32"))]
    snapshot::add(&mut app);
    app.run();
}

/// `GE_SCREENSHOT=out.png` opens a screen, plays a few notes by itself, saves a screenshot and
/// exits, to check the screens without a person at the instrument:
///
/// - `GE_SCREEN`: menu, drills, drill, learn, find, settings or stats;
/// - `GE_LEVEL`: the drill level;
/// - `GE_PLAY`: the notes played right (heard by the "microphone"), `GE_TAP` tapped right,
///   `GE_MISS` wrong notes heard;
/// - `GE_WAIT`: seconds to wait after them, for the hints;
/// - `GE_SIZE`: the window size, `1280x860` by default.
///
/// The microphone is off. Use `XDG_CONFIG_HOME` to keep the saves away from the real ones.
#[cfg(not(target_arch = "wasm32"))]
mod snapshot {
    use bevy::prelude::*;
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    use bevy::window::{PrimaryWindow, WindowResolution};
    use guitar_exercises::audio::NoteHeard;
    use guitar_exercises::fretboard::FretTapped;
    use guitar_exercises::screens::Expected;
    use guitar_exercises::screens::drills::SelectedLevel;
    use guitar_exercises::settings::Settings;
    use guitar_exercises::ui::Screen;

    #[derive(Resource)]
    struct Snapshot {
        path: String,
        screen: Screen,
        play: u32,
        tap: u32,
        miss: u32,
        wait: f32,
    }

    fn var<T: std::str::FromStr>(name: &str, default: T) -> T {
        std::env::var(name).ok().and_then(|v| v.parse().ok()).unwrap_or(default)
    }

    pub fn add(app: &mut App) {
        let Ok(path) = std::env::var("GE_SCREENSHOT") else {
            return;
        };
        let screen = match std::env::var("GE_SCREEN").unwrap_or_default().as_str() {
            "drills" => Screen::Drills,
            "drill" => Screen::Drill,
            "learn" => Screen::Learn,
            "find" => Screen::Find,
            "settings" => Screen::Settings,
            "stats" => Screen::Stats,
            _ => Screen::Menu,
        };
        app.insert_resource(Snapshot {
            path,
            screen,
            play: var("GE_PLAY", 0),
            tap: var("GE_TAP", 0),
            miss: var("GE_MISS", 0),
            wait: var("GE_WAIT", 0.0),
        })
        .insert_resource(SelectedLevel(var("GE_LEVEL", 0)))
        .add_systems(Startup, setup)
        .add_systems(Update, run);
    }

    fn setup(mut window: Single<&mut Window, With<PrimaryWindow>>, mut settings: ResMut<Settings>) {
        // A window in the background is throttled by the compositor with vsync.
        window.present_mode = bevy::window::PresentMode::AutoNoVsync;
        let size = std::env::var("GE_SIZE").unwrap_or_default();
        if let Some((w, h)) = size.split_once('x').and_then(|(w, h)| Some((w.parse::<u32>().ok()?, h.parse::<u32>().ok()?))) {
            window.resolution = WindowResolution::new(w, h);
        }
        settings.microphone = false;
    }

    #[allow(clippy::too_many_arguments)]
    fn run(
        mut frame: Local<u32>,
        mut sent: Local<u32>,
        mut done_at: Local<Option<f32>>,
        mut shot: Local<Option<u32>>,
        time: Res<Time<Real>>,
        snapshot: Res<Snapshot>,
        expected: Res<Expected>,
        mut next: ResMut<NextState<Screen>>,
        mut heard: MessageWriter<NoteHeard>,
        mut tapped: MessageWriter<FretTapped>,
        mut commands: Commands,
        mut exit: MessageWriter<AppExit>,
    ) {
        *frame += 1;
        if *frame == 5 {
            next.set(snapshot.screen);
        }
        let start = 30;
        let total = snapshot.play + snapshot.tap + snapshot.miss;
        // One answer every 8 frames; the note games ignore the answers while they show the
        // previous one.
        if *frame > start && (*frame - start).is_multiple_of(8) && *sent < total {
            let i = *sent;
            if i < snapshot.play {
                if let Some(midi) = expected.midi {
                    heard.write(NoteHeard { midi, cents: 0.0 });
                }
            } else if i < snapshot.play + snapshot.tap {
                if let Some(position) = expected.position {
                    tapped.write(FretTapped(position));
                }
            } else if let Some(midi) = expected.midi {
                heard.write(NoteHeard { midi: midi + 1, cents: 0.0 });
            }
            *sent += 1;
        }
        if *frame > start && *sent >= total && done_at.is_none() {
            *done_at = Some(time.elapsed_secs());
        }
        if let Some(at) = *done_at
            && shot.is_none()
            && time.elapsed_secs() - at >= snapshot.wait + 0.5
        {
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(snapshot.path.clone()));
            *shot = Some(*frame);
        }
        if shot.is_some_and(|f| *frame > f + 20) {
            exit.write(AppExit::Success);
        }
    }
}
