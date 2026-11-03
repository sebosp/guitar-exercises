//! The songs of the player: the Guitar Pro files imported, saved on the device (the bytes as
//! base64 text, with the other saves), with the track chosen for each.
//!
//! A file comes in dropped on the window (the desktops and the browser), from the open dialog
//! (the desktops and the browser), or copied to the `songs` folder of the saves (every device
//! with files, the way in on Android): the folder is read when the songs page opens.
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::song::{self, SongData};
use crate::ui::Screen;

/// A song as saved in the index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SavedSong {
    pub id: String,
    pub file_name: String,
    pub track: usize,
}

#[derive(Debug, Clone)]
pub struct LibrarySong {
    pub saved: SavedSong,
    pub data: SongData,
}

#[derive(Resource, Default)]
pub struct Library {
    pub songs: Vec<LibrarySong>,
    /// The last import or error, for the songs page.
    pub notice: String,
}

/// Files to import: their name and their bytes.
#[derive(Resource, Default)]
pub struct Incoming(pub Vec<(String, Vec<u8>)>);

/// The open dialog is wanted.
#[derive(Resource, Default)]
pub struct OpenDialog(pub bool);

/// The id of a file: a hash of its bytes, the same file twice is one song.
pub fn id_of(bytes: &[u8]) -> String {
    // FNV-1a, 64 bits.
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ *b as u64).wrapping_mul(0x0100_0000_01b3));
    format!("{hash:016x}")
}

impl Library {
    fn load() -> Self {
        let index: Vec<SavedSong> = crate::storage::load("songs").unwrap_or_default();
        let songs = index
            .into_iter()
            .filter_map(|saved| {
                let text: String = crate::storage::load(&format!("song-{}", saved.id))?;
                let bytes = STANDARD.decode(text).ok()?;
                let data = song::read(&saved.file_name, &bytes).ok()?;
                Some(LibrarySong { saved, data })
            })
            .collect();
        Library { songs, notice: String::new() }
    }

    fn save_index(&self) {
        let index: Vec<&SavedSong> = self.songs.iter().map(|s| &s.saved).collect();
        crate::storage::save("songs", &index);
    }

    /// A file read and saved, the track for the instrument chosen.
    pub fn import(&mut self, file_name: &str, bytes: &[u8], kind: crate::music::InstrumentKind) {
        let id = id_of(bytes);
        if self.songs.iter().any(|s| s.saved.id == id) {
            self.notice = format!("{file_name} is already in your songs.");
            return;
        }
        match song::read(file_name, bytes) {
            Ok(data) if data.tracks.is_empty() => self.notice = format!("{file_name} has no track of strings to play."),
            Ok(data) => {
                crate::storage::save(&format!("song-{id}"), &STANDARD.encode(bytes));
                let track = data.track_for(kind);
                self.notice = format!("Added {}.", data.title);
                self.songs.push(LibrarySong { saved: SavedSong { id, file_name: file_name.to_string(), track }, data });
                self.save_index();
            }
            Err(err) => self.notice = err,
        }
    }

    pub fn remove(&mut self, id: &str) {
        self.songs.retain(|s| s.saved.id != id);
        crate::storage::remove(&format!("song-{id}"));
        self.save_index();
    }

    pub fn set_track(&mut self, id: &str, track: usize) {
        if let Some(song) = self.songs.iter_mut().find(|s| s.saved.id == id) {
            song.saved.track = track.min(song.data.tracks.len().saturating_sub(1));
        }
        self.save_index();
    }

    pub fn get(&self, id: &str) -> Option<&LibrarySong> {
        self.songs.iter().find(|s| s.saved.id == id)
    }
}

/// The key of a section of a song for the progress and the drill: `song/<id>/t<track>/m<a>-<b>`.
pub fn section_key(id: &str, track: usize, measures: &std::ops::Range<usize>) -> String {
    format!("song/{id}/t{track}/m{}-{}", measures.start, measures.end)
}

/// The song, the track and the measures of a key.
pub fn parse_key(key: &str) -> Option<(&str, usize, std::ops::Range<usize>)> {
    let mut parts = key.strip_prefix("song/")?.split('/');
    let id = parts.next()?;
    let track = parts.next()?.strip_prefix('t')?.parse().ok()?;
    let (a, b) = parts.next()?.strip_prefix('m')?.split_once('-')?;
    Some((id, track, a.parse().ok()?..b.parse().ok()?))
}

pub struct LibraryPlugin;

impl Plugin for LibraryPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Library::load())
            .init_resource::<Incoming>()
            .init_resource::<OpenDialog>()
            .add_systems(Startup, web::listen_for_drops)
            .add_systems(OnEnter(Screen::Songs), read_folder)
            .add_systems(Update, (dropped, web::take_files, open_dialog, import).chain());
    }
}

fn import(
    mut incoming: ResMut<Incoming>,
    mut library: ResMut<Library>,
    settings: Res<crate::settings::Settings>,
    screen: Res<State<Screen>>,
    mut next: ResMut<NextState<Screen>>,
) {
    if incoming.0.is_empty() {
        return;
    }
    for (name, bytes) in std::mem::take(&mut incoming.0) {
        library.import(&name, &bytes, settings.instrument);
    }
    // To the songs page, built again with the new ones.
    if *screen.get() != Screen::Drill {
        next.set(Screen::Songs);
    }
}

/// A file dropped on the window of a desktop.
fn dropped(mut drops: MessageReader<bevy::window::FileDragAndDrop>, mut incoming: ResMut<Incoming>, mut library: ResMut<Library>) {
    for drop in drops.read() {
        if let bevy::window::FileDragAndDrop::DroppedFile { path_buf, .. } = drop {
            let name = path_buf.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            match std::fs::read(path_buf) {
                Ok(bytes) => incoming.0.push((name, bytes)),
                Err(err) => library.notice = format!("Unable to read {name}: {err}"),
            }
        }
    }
}

/// The Guitar Pro files of the songs folder not imported yet.
fn read_folder(mut incoming: ResMut<Incoming>, library: Res<Library>) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let Some(folder) = crate::storage::songs_folder() else { return };
        let _ = std::fs::create_dir_all(&folder);
        let Ok(entries) = std::fs::read_dir(&folder) else { return };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let ext = name.rsplit('.').next().unwrap_or_default().to_lowercase();
            if !["gp", "gpx", "gp5", "gp4", "gp3"].contains(&ext.as_str()) {
                continue;
            }
            if let Ok(bytes) = std::fs::read(&path)
                && library.get(&id_of(&bytes)).is_none()
            {
                incoming.0.push((name, bytes));
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = (&mut incoming, &library);
    }
}

/// The open dialog of the desktops, on the main thread (macOS wants it there); in the browser
/// it is a file input, the file comes back later.
fn open_dialog(mut wanted: ResMut<OpenDialog>, mut incoming: ResMut<Incoming>, _main: bevy::ecs::system::NonSendMarker) {
    if !std::mem::take(&mut wanted.0) {
        return;
    }
    #[cfg(not(any(target_arch = "wasm32", target_os = "android")))]
    if let Some(path) = rfd::FileDialog::new().add_filter("Guitar Pro", &["gp", "gpx", "gp5", "gp4", "gp3"]).pick_file()
        && let Ok(bytes) = std::fs::read(&path)
    {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        incoming.0.push((name, bytes));
    }
    #[cfg(target_arch = "wasm32")]
    {
        let _ = &mut incoming;
        web::pick();
    }
    #[cfg(target_os = "android")]
    {
        let _ = &mut incoming;
    }
}

/// Whether the platform has the open dialog.
pub const HAS_DIALOG: bool = cfg!(not(target_os = "android"));

#[cfg(not(target_arch = "wasm32"))]
mod web {
    use super::Incoming;
    use bevy::prelude::*;

    pub fn listen_for_drops() {}

    pub fn take_files(_: ResMut<Incoming>) {}
}

/// The files dropped on the page or picked in the browser come in asynchronously: a queue read
/// every frame.
#[cfg(target_arch = "wasm32")]
mod web {
    use std::cell::RefCell;

    use bevy::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;

    use super::Incoming;

    thread_local! {
        static FILES: RefCell<Vec<(String, Vec<u8>)>> = const { RefCell::new(Vec::new()) };
    }

    fn read_file(file: web_sys::File) {
        let name = file.name();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(buffer) = wasm_bindgen_futures::JsFuture::from(file.array_buffer()).await {
                let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
                FILES.with_borrow_mut(|files| files.push((name, bytes)));
            }
        });
    }

    /// The page takes the dragged files instead of opening them.
    pub fn listen_for_drops() {
        let Some(document) = web_sys::window().and_then(|w| w.document()) else { return };
        let over = Closure::<dyn Fn(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| event.prevent_default());
        let drop = Closure::<dyn Fn(web_sys::DragEvent)>::new(|event: web_sys::DragEvent| {
            event.prevent_default();
            let Some(files) = event.data_transfer().and_then(|d| d.files()) else { return };
            for i in 0..files.length() {
                if let Some(file) = files.get(i) {
                    read_file(file);
                }
            }
        });
        let _ = document.add_event_listener_with_callback("dragover", over.as_ref().unchecked_ref());
        let _ = document.add_event_listener_with_callback("drop", drop.as_ref().unchecked_ref());
        over.forget();
        drop.forget();
    }

    pub fn pick() {
        wasm_bindgen_futures::spawn_local(async {
            if let Some(file) = rfd::AsyncFileDialog::new().add_filter("Guitar Pro", &["gp", "gpx", "gp5", "gp4", "gp3"]).pick_file().await
            {
                let bytes = file.read().await;
                FILES.with_borrow_mut(|files| files.push((file.file_name(), bytes)));
            }
        });
    }

    pub fn take_files(mut incoming: ResMut<Incoming>) {
        FILES.with_borrow_mut(|files| incoming.0.append(files));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys() {
        let key = section_key("00ff", 2, &(4..12));
        assert_eq!(key, "song/00ff/t2/m4-12");
        assert_eq!(parse_key(&key), Some(("00ff", 2, 4..12)));
        assert_eq!(parse_key("fingers/0/1-2-3-4"), None);
        assert_eq!(id_of(b"a"), id_of(b"a"));
        assert_ne!(id_of(b"a"), id_of(b"b"));
    }
}
