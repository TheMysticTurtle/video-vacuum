//! The outside programs Video Vacuum runs, shipped inside the app (D-32):
//! yt-dlp (downloads from YouTube), Deno (runs YouTube's player code for
//! yt-dlp) and a small FFmpeg build (joins video + audio, makes MP3s).
//!
//! - They live in one folder, `<install dir>\tools\`, which the installer
//!   fills. yt-dlp finds ffmpeg/ffprobe/deno next to itself; the paths are
//!   also passed explicitly so nothing on the user's PATH can stand in.
//! - `tools.json` (compiled in) pins each tool's version, official download
//!   and SHA-256. The version is also the minimum the app accepts.
//! - If a tool is missing or too old, `restore` downloads that exact pinned
//!   file, checks the SHA-256 and puts it in place. Windows' own curl.exe
//!   and tar.exe do the download and unzip (both ship with Windows 10/11).
//! - "Update yt-dlp" runs yt-dlp's own updater (`yt-dlp -U`) on the bundled
//!   copy; it checks yt-dlp's official checksums itself.

use std::ffi::OsStr;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Windows: don't flash a console window for each background call.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const PINS_JSON: &str = include_str!("../tools.json");

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolPin {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub repo: String,
    pub license: String,
    /// The bundled version, and the oldest one the app accepts.
    pub version: String,
    /// The programs this tool puts in the tools folder.
    pub files: Vec<String>,
    pub version_args: Vec<String>,
    pub download: Download,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Download {
    pub url: String,
    pub sha256: String,
    /// "exe" (the file itself) or "zip" (contains `files`).
    pub kind: String,
}

#[derive(Deserialize)]
struct PinsFile {
    tools: Vec<ToolPin>,
}

/// The pinned tools, in display order (yt-dlp, Deno, FFmpeg).
pub fn pins() -> Vec<ToolPin> {
    serde_json::from_str::<PinsFile>(PINS_JSON).expect("tools.json is valid (checked by a test)").tools
}

/// What the page shows about one tool.
#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub repo: String,
    pub license: String,
    pub required: String,
    pub installed: Option<String>,
    /// "ok", "missing" or "outdated".
    pub state: &'static str,
}

/// What the page shows about the tools.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsStatus {
    pub ready: bool,
    pub tools: Vec<ToolInfo>,
    pub ytdlp_version: Option<String>,
}

/// The tools folder. Cloned into each download (just a path).
#[derive(Debug, Clone, Default)]
pub struct Tools {
    pub dir: PathBuf,
}

impl Tools {
    pub fn new(dir: PathBuf) -> Self {
        Tools { dir }
    }

    pub fn path(&self, file: &str) -> PathBuf {
        self.dir.join(file)
    }

    pub fn ytdlp(&self) -> PathBuf {
        self.path("yt-dlp.exe")
    }

    /// Runs each tool's version command (about 1 s, mostly yt-dlp starting up)
    /// and compares it with the pinned minimum.
    pub fn status(&self) -> ToolsStatus {
        let tools: Vec<ToolInfo> = pins().iter().map(|pin| self.check(pin)).collect();
        let ytdlp_version = tools.iter().find(|t| t.id == "yt-dlp").and_then(|t| t.installed.clone());
        ToolsStatus { ready: tools.iter().all(|t| t.state == "ok"), tools, ytdlp_version }
    }

    fn check(&self, pin: &ToolPin) -> ToolInfo {
        let present = pin.files.iter().all(|file| self.path(file).is_file());
        let installed = if present { self.version_of(pin) } else { None };
        let state = match &installed {
            None => "missing",
            Some(version) if !at_least(version, &pin.version) => "outdated",
            Some(_) => "ok",
        };
        ToolInfo {
            id: pin.id.clone(),
            name: pin.name.clone(),
            purpose: pin.purpose.clone(),
            repo: pin.repo.clone(),
            license: pin.license.clone(),
            required: pin.version.clone(),
            installed,
            state,
        }
    }

    fn version_of(&self, pin: &ToolPin) -> Option<String> {
        let output = hidden_command(&self.path(&pin.files[0])).args(&pin.version_args).output().ok()?;
        parse_version(&String::from_utf8_lossy(&output.stdout))
    }

    /// yt-dlp is told exactly where ffmpeg and deno are (the bundled copies).
    pub fn ytdlp_base_args(&self) -> Vec<String> {
        vec![
            "--ignore-config".into(), // the user's yt-dlp config can't change our results
            "--no-playlist".into(),   // a `&list=` link means just that one video
            "--encoding".into(),
            "utf-8".into(), // emoji and non-Latin titles survive the pipe
            "--ffmpeg-location".into(),
            self.dir.display().to_string(),
            "--js-runtimes".into(),
            format!("deno:{}", self.path("deno.exe").display()),
        ]
    }

    pub fn ytdlp_command(&self) -> Result<Command, String> {
        let path = self.ytdlp();
        if !path.is_file() {
            return Err("yt-dlp is missing. Use \"Restore\" in the banner at the top, or reinstall Video Vacuum.".into());
        }
        Ok(hidden_command(&path))
    }

    pub fn run_ytdlp<I, S>(&self, args: I) -> Result<Output, String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.ytdlp_command()?
            .args(args)
            .output()
            .map_err(|err| format!("Couldn't start yt-dlp: {err}"))
    }

    /// yt-dlp's own updater on the bundled copy. It leaves the previous copy
    /// as `yt-dlp.exe.old`, which is removed here. Returns its output.
    pub fn update_ytdlp(&self) -> Result<String, String> {
        let output = self.run_ytdlp(["-U"])?;
        let _ = fs::remove_file(self.path("yt-dlp.exe.old"));
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        if output.status.success() {
            Ok(text.trim().to_string())
        } else {
            Err(text.trim().to_string())
        }
    }

    /// Puts back the pinned copy of every tool that's missing or too old.
    /// `note` is told each step (for the diagnostic log).
    pub fn restore(&self, note: &dyn Fn(String)) -> Result<Vec<String>, String> {
        fs::create_dir_all(&self.dir).map_err(|err| format!("Couldn't use {}: {err}", self.dir.display()))?;
        let mut restored = Vec::new();
        for pin in pins() {
            if self.check(&pin).state == "ok" {
                continue;
            }
            note(format!("Restoring {} {} from {}", pin.name, pin.version, pin.download.url));
            self.restore_one(&pin)?;
            note(format!("{} {} restored and its checksum verified", pin.name, pin.version));
            restored.push(pin.name.clone());
        }
        Ok(restored)
    }

    fn restore_one(&self, pin: &ToolPin) -> Result<(), String> {
        let work = self.dir.join(format!(".restore-{}", pin.id));
        let _ = fs::remove_dir_all(&work);
        fs::create_dir_all(&work).map_err(|err| err.to_string())?;
        let result = (|| {
            let download = work.join("download");
            fetch(&pin.download.url, &download)?;
            verify_sha256(&download, &pin.download.sha256)?;
            let source_dir = if pin.download.kind == "zip" {
                let unpacked = work.join("unpacked");
                fs::create_dir_all(&unpacked).map_err(|err| err.to_string())?;
                unzip(&download, &unpacked)?;
                unpacked
            } else {
                fs::rename(&download, work.join(&pin.files[0])).map_err(|err| err.to_string())?;
                work.clone()
            };
            for file in &pin.files {
                let from = find_file(&source_dir, file).ok_or(format!("{file} wasn't in the download"))?;
                let to = self.path(file);
                let _ = fs::remove_file(&to);
                fs::rename(&from, &to)
                    .or_else(|_| fs::copy(&from, &to).map(|_| ()))
                    .map_err(|err| format!("Couldn't put {file} in place: {err}"))?;
            }
            Ok(())
        })();
        let _ = fs::remove_dir_all(&work);
        result
    }
}

pub fn hidden_command(program: &Path) -> Command {
    #[allow(unused_mut)]
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    command
}

/// Windows' own programs, by full path (never whatever is first on PATH).
fn system32(program: &str) -> PathBuf {
    let root = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    root.join("System32").join(program)
}

/// Downloads over HTTPS only, failing on any HTTP error.
fn fetch(url: &str, to: &Path) -> Result<(), String> {
    let output = hidden_command(&system32("curl.exe"))
        .args(["--fail", "--silent", "--show-error", "--location", "--proto", "=https", "--retry", "3", "--output"])
        .arg(to)
        .arg(url)
        .output()
        .map_err(|err| format!("Couldn't start the download: {err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Download failed: {}", String::from_utf8_lossy(&output.stderr).trim()))
    }
}

fn unzip(zip: &Path, into: &Path) -> Result<(), String> {
    let output = hidden_command(&system32("tar.exe"))
        .arg("-xf")
        .arg(zip)
        .arg("-C")
        .arg(into)
        .output()
        .map_err(|err| format!("Couldn't unpack the download: {err}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("Couldn't unpack the download: {}", String::from_utf8_lossy(&output.stderr).trim()))
    }
}

/// Fails unless the file's SHA-256 matches the pinned one.
pub fn verify_sha256(file: &Path, expected: &str) -> Result<(), String> {
    let mut reader = fs::File::open(file).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = reader.read(&mut buffer).map_err(|err| err.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    if actual.eq_ignore_ascii_case(expected.trim()) {
        Ok(())
    } else {
        Err(format!("The download didn't match its checksum (expected {expected}, got {actual}); it wasn't used."))
    }
}

/// Finds `name` in `dir` or its subfolders (zips may have a top folder).
fn find_file(dir: &Path, name: &str) -> Option<PathBuf> {
    for entry in fs::read_dir(dir).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_file(&path, name) {
                return Some(found);
            }
        } else if entry.file_name().to_string_lossy().eq_ignore_ascii_case(name) {
            return Some(path);
        }
    }
    None
}

/// The first "1.2.3"-style number in a version line:
/// "2026.11.23", "deno 2.9.8 (stable…)", "ffmpeg version 9.1 Copyright…".
pub fn parse_version(text: &str) -> Option<String> {
    text.split_whitespace()
        .map(|word| word.trim_start_matches(['v', 'n']))
        .find(|word| word.contains('.') && word.split('.').all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit())))
        .map(str::to_string)
}

/// True if `version` is the same as or newer than `minimum` (numeric, part by part).
pub fn at_least(version: &str, minimum: &str) -> bool {
    let parts = |v: &str| v.split('.').map(|p| p.parse::<u64>().unwrap_or(0)).collect::<Vec<_>>();
    let (a, b) = (parts(version), parts(minimum));
    for i in 0..a.len().max(b.len()) {
        let (x, y) = (a.get(i).copied().unwrap_or(0), b.get(i).copied().unwrap_or(0));
        if x != y {
            return x > y;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_pins_file_is_complete() {
        let pins = pins();
        assert_eq!(pins.iter().map(|p| p.id.as_str()).collect::<Vec<_>>(), ["yt-dlp", "deno", "ffmpeg"]);
        for pin in &pins {
            assert!(pin.download.url.starts_with("https://"), "{}", pin.id);
            assert!(["exe", "zip"].contains(&pin.download.kind.as_str()), "{}", pin.id);
            assert!(!pin.files.is_empty() && parse_version(&pin.version).is_some(), "{}", pin.id);
            assert!(pin.repo.starts_with("https://"), "{}", pin.id);
            let sha = &pin.download.sha256;
            assert!(sha.len() == 64 && sha.chars().all(|c| c.is_ascii_hexdigit()), "{}: pinned SHA-256", pin.id);
        }
    }

    #[test]
    fn versions_are_read_from_each_tools_output() {
        assert_eq!(parse_version("2026.11.23\n").as_deref(), Some("2026.11.23"));
        assert_eq!(parse_version("deno 2.9.8 (stable, release, x86_64-pc-windows-msvc)").as_deref(), Some("2.9.8"));
        assert_eq!(parse_version("ffmpeg version 9.1 Copyright (c) 2000-2026").as_deref(), Some("9.1"));
        assert_eq!(parse_version("ffmpeg version n9.1.2 Copyright").as_deref(), Some("9.1.2"));
        assert_eq!(parse_version("nothing here"), None);
    }

    #[test]
    fn versions_compare_part_by_part() {
        assert!(at_least("2026.11.23", "2026.11.23"));
        assert!(at_least("2026.12.1", "2026.11.23"));
        assert!(at_least("2026.11.23.235959", "2026.11.23"));
        assert!(!at_least("2026.10.26", "2026.11.23"));
        assert!(at_least("2.10.0", "2.9.8"), "10 > 9 as numbers, not text");
        assert!(!at_least("9.0", "9.1"));
    }

    #[test]
    fn checksums_are_enforced() {
        let file = std::env::temp_dir().join(format!("video-vacuum-sha-{}", std::process::id()));
        fs::write(&file, b"abc").unwrap();
        let abc = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
        assert!(verify_sha256(&file, abc).is_ok());
        assert!(verify_sha256(&file, &abc.to_uppercase()).is_ok());
        assert!(verify_sha256(&file, &"0".repeat(64)).unwrap_err().contains("checksum"));
        let _ = fs::remove_file(file);
    }

    #[test]
    fn a_missing_folder_reports_every_tool_missing() {
        let tools = Tools::new(std::env::temp_dir().join("video-vacuum-no-tools-here"));
        let status = tools.status();
        assert!(!status.ready);
        assert!(status.tools.iter().all(|t| t.state == "missing" && t.installed.is_none()));
        assert!(tools.ytdlp_command().unwrap_err().contains("Restore"));
    }

    #[test]
    fn yt_dlp_gets_the_bundled_paths() {
        let tools = Tools::new(PathBuf::from(r"C:\App\tools"));
        let args = tools.ytdlp_base_args();
        assert!(args.windows(2).any(|w| w == ["--ffmpeg-location", r"C:\App\tools"]));
        assert!(args.windows(2).any(|w| w[0] == "--js-runtimes" && w[1].ends_with(r"tools\deno.exe")));
    }

    #[test]
    fn files_are_found_in_zip_subfolders() {
        let dir = std::env::temp_dir().join(format!("video-vacuum-find-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(dir.join("top").join("bin")).unwrap();
        fs::write(dir.join("top").join("bin").join("FFmpeg.exe"), b"x").unwrap();
        assert_eq!(find_file(&dir, "ffmpeg.exe"), Some(dir.join("top").join("bin").join("FFmpeg.exe")));
        assert_eq!(find_file(&dir, "deno.exe"), None);
        let _ = fs::remove_dir_all(dir);
    }
}
