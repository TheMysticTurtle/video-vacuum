//! Download history and settings, kept as two small JSON files in the app's
//! own data folder (`%APPDATA%\com.themysticturtle.videovacuum` on Windows).
//!
//! - Saves are atomic: write a temp file, then rename it over the real one,
//!   so a crash never leaves a half-written file.
//! - Each file has a format version. A file from a newer app version is
//!   never overwritten; the app just doesn't save to it.

use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

const HISTORY_VERSION: u32 = 1;
const SETTINGS_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: String,
    pub title: String,
    pub video_url: String,
    pub channel: String,
    pub thumbnail: Option<String>,
    /// "MP4", "M4A"…
    pub format: String,
    /// "1080p", "Best available", "Audio"
    pub quality: String,
    pub file_path: String,
    pub size_bytes: Option<u64>,
    /// Seconds since 1970 (UTC).
    pub downloaded_at: u64,
}

#[derive(Debug, Deserialize)]
struct HistoryFile {
    version: u32,
    entries: Vec<HistoryEntry>,
}

/// The same shape for saving, borrowing the list instead of copying it.
#[derive(Serialize)]
struct HistoryFileRef<'a> {
    version: u32,
    entries: &'a [HistoryEntry],
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    /// None = the system Downloads folder.
    pub default_folder: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SettingsFile {
    version: u32,
    #[serde(flatten)]
    settings: Settings,
}

pub struct Store {
    dir: PathBuf,
    /// Newest first.
    pub history: Vec<HistoryEntry>,
    pub settings: Settings,
    history_writable: bool,
    settings_writable: bool,
}

impl Store {
    pub fn open(dir: PathBuf) -> Self {
        let (history, history_writable) =
            match read_versioned::<HistoryFile>(&dir.join("history.json"), HISTORY_VERSION, |f| f.version) {
                Loaded::Ok(file) => (file.entries, true),
                Loaded::Missing => (Vec::new(), true),
                Loaded::Newer | Loaded::Unreadable => (Vec::new(), false),
            };
        let (settings, settings_writable) =
            match read_versioned::<SettingsFile>(&dir.join("settings.json"), SETTINGS_VERSION, |f| f.version) {
                Loaded::Ok(file) => (file.settings, true),
                Loaded::Missing => (Settings::default(), true),
                Loaded::Newer | Loaded::Unreadable => (Settings::default(), false),
            };
        Store { dir, history, settings, history_writable, settings_writable }
    }

    pub fn add(&mut self, entry: HistoryEntry) -> Result<(), String> {
        self.history.insert(0, entry);
        self.save_history()
    }

    pub fn find(&self, id: &str) -> Option<&HistoryEntry> {
        self.history.iter().find(|entry| entry.id == id)
    }

    pub fn set_default_folder(&mut self, folder: Option<String>) -> Result<(), String> {
        self.settings.default_folder = folder;
        if !self.settings_writable {
            return Err("Settings were saved by a newer version of Video Vacuum, so they weren't changed.".into());
        }
        let file = SettingsFile { version: SETTINGS_VERSION, settings: self.settings.clone() };
        write_atomic(&self.dir.join("settings.json"), &file)
    }

    fn save_history(&self) -> Result<(), String> {
        if !self.history_writable {
            return Err("History was saved by a newer version of Video Vacuum, so it wasn't changed.".into());
        }
        let file = HistoryFileRef { version: HISTORY_VERSION, entries: &self.history };
        write_atomic(&self.dir.join("history.json"), &file)
    }
}

enum Loaded<T> {
    Ok(T),
    Missing,
    Newer,
    /// Damaged or unreadable: start empty and don't overwrite it, so it can be recovered by hand.
    Unreadable,
}

fn read_versioned<T: DeserializeOwned>(path: &Path, current: u32, version_of: impl Fn(&T) -> u32) -> Loaded<T> {
    let bytes = match fs::read(path) {
        Ok(bytes) => bytes,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Loaded::Missing,
        Err(_) => return Loaded::Unreadable,
    };
    // Check the version first, so a newer file with fields we don't know is recognized as newer.
    #[derive(Deserialize)]
    struct VersionOnly {
        version: u32,
    }
    match serde_json::from_slice::<VersionOnly>(&bytes) {
        Ok(v) if v.version > current => return Loaded::Newer,
        Ok(_) => {}
        Err(_) => return Loaded::Unreadable,
    }
    match serde_json::from_slice::<T>(&bytes) {
        Ok(file) if version_of(&file) <= current => Loaded::Ok(file),
        Ok(_) => Loaded::Newer,
        Err(_) => Loaded::Unreadable,
    }
}

fn write_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    let dir = path.parent().ok_or("Bad data folder.")?;
    fs::create_dir_all(dir).map_err(|err| format!("Couldn't create {}: {err}", dir.display()))?;
    let temp = path.with_extension("json.tmp");
    let json = serde_json::to_vec_pretty(value).map_err(|err| err.to_string())?;
    fs::write(&temp, json).map_err(|err| format!("Couldn't save {}: {err}", temp.display()))?;
    // std::fs::rename replaces an existing file on Windows too (MoveFileExW + REPLACE_EXISTING).
    fs::rename(&temp, path).map_err(|err| format!("Couldn't save {}: {err}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("video-vacuum-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn entry(id: &str) -> HistoryEntry {
        HistoryEntry {
            id: id.into(),
            title: "Me at the zoo".into(),
            video_url: "https://www.youtube.com/watch?v=jNQXAC9IVRw".into(),
            channel: "jawed".into(),
            thumbnail: None,
            format: "MP4".into(),
            quality: "240p".into(),
            file_path: r"C:\Videos\Me at the zoo.mp4".into(),
            size_bytes: Some(791_000),
            downloaded_at: 1_700_000_000,
        }
    }

    #[test]
    fn history_and_settings_survive_a_restart() {
        let dir = temp_dir("roundtrip");
        let mut store = Store::open(dir.clone());
        store.add(entry("a")).unwrap();
        store.add(entry("b")).unwrap();
        store.set_default_folder(Some(r"D:\Clips".into())).unwrap();

        let reopened = Store::open(dir.clone());
        assert_eq!(reopened.history.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), ["b", "a"]);
        assert_eq!(reopened.settings.default_folder.as_deref(), Some(r"D:\Clips"));
        assert!(!dir.join("history.json.tmp").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn newer_or_damaged_files_are_never_overwritten() {
        let dir = temp_dir("newer");
        fs::create_dir_all(&dir).unwrap();
        let newer = r#"{"version": 99, "entries": [], "somethingNew": true}"#;
        fs::write(dir.join("history.json"), newer).unwrap();
        fs::write(dir.join("settings.json"), "not json").unwrap();

        let mut store = Store::open(dir.clone());
        assert!(store.add(entry("a")).is_err());
        assert!(store.set_default_folder(None).is_err());
        assert_eq!(fs::read_to_string(dir.join("history.json")).unwrap(), newer);
        assert_eq!(fs::read_to_string(dir.join("settings.json")).unwrap(), "not json");
        let _ = fs::remove_dir_all(dir);
    }
}
