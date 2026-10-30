//! Local saves: JSON files in the configuration directory of the platform (the private storage
//! of the app on Android), the `localStorage` of the browser on the web.
use bevy::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// Reads a saved value, `None` when there is none or it is unreadable.
pub fn load<T: DeserializeOwned>(name: &str) -> Option<T> {
    let json = backend::read(name)?;
    match serde_json::from_str(&json) {
        Ok(value) => Some(value),
        Err(err) => {
            warn!("Ignoring the unreadable {name}: {err}");
            None
        }
    }
}

pub fn save<T: Serialize>(name: &str, value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => backend::write(name, &json),
        Err(err) => error!("Unable to serialize the {name}: {err}"),
    }
}

pub fn now_unix_seconds() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use std::path::PathBuf;

    use bevy::prelude::*;

    pub fn config_dir() -> Option<PathBuf> {
        // The private storage of the app.
        #[cfg(target_os = "android")]
        {
            return bevy::android::ANDROID_APP.get().and_then(|app| app.internal_data_path());
        }
        #[cfg(target_os = "ios")]
        {
            return std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support/guitar-exercises"));
        }
        #[cfg(target_os = "macos")]
        {
            return std::env::var_os("HOME").map(|home| PathBuf::from(home).join("Library/Application Support/guitar-exercises"));
        }
        #[cfg(target_os = "windows")]
        {
            return std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("guitar-exercises"));
        }
        #[allow(unreachable_code)]
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("guitar-exercises"))
    }

    pub fn read(name: &str) -> Option<String> {
        std::fs::read_to_string(config_dir()?.join(format!("{name}.json"))).ok()
    }

    pub fn write(name: &str, json: &str) {
        let Some(dir) = config_dir() else {
            warn!("No configuration directory, the {name} is not saved");
            return;
        };
        let path = dir.join(format!("{name}.json"));
        // A temporary file first, the save survives a crash while writing.
        let tmp = dir.join(format!("{name}.json.tmp"));
        let result = std::fs::create_dir_all(&dir).and_then(|_| std::fs::write(&tmp, json)).and_then(|_| std::fs::rename(&tmp, &path));
        if let Err(err) = result {
            error!("Unable to save {}: {err}", path.display());
        }
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    fn local_storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    pub fn read(name: &str) -> Option<String> {
        local_storage()?.get_item(&format!("guitar-exercises.{name}")).ok().flatten()
    }

    pub fn write(name: &str, json: &str) {
        if let Some(storage) = local_storage() {
            let _ = storage.set_item(&format!("guitar-exercises.{name}"), json);
        }
    }
}
