//! Finding and running the outside programs Video Vacuum needs:
//! yt-dlp (the downloader), ffmpeg (joins video + audio) and deno (the
//! JavaScript runtime yt-dlp needs for full YouTube support).
//!
//! All three come from one winget package: `winget install --id yt-dlp.yt-dlp`
//! pulls in `yt-dlp.FFmpeg` and `DenoLand.Deno` as dependencies.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde::Serialize;

/// Windows: don't flash a console window for each background call.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
/// Windows: give winget its own visible console so the user can answer its prompts.
#[cfg(windows)]
const CREATE_NEW_CONSOLE: u32 = 0x0000_0010;

/// Full paths to the tools that were found.
#[derive(Debug, Clone, Default)]
pub struct Tools {
    pub ytdlp: Option<PathBuf>,
    pub ffmpeg: Option<PathBuf>,
    pub deno: Option<PathBuf>,
}

/// What the page shows about the tools.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsStatus {
    pub ready: bool,
    pub missing: Vec<String>,
    pub ytdlp_version: Option<String>,
    pub can_install: bool,
}

impl Tools {
    pub fn find() -> Self {
        let dirs = search_dirs();
        Tools {
            ytdlp: find_in(&dirs, "yt-dlp"),
            ffmpeg: find_in(&dirs, "ffmpeg"),
            deno: find_in(&dirs, "deno"),
        }
    }

    pub fn missing(&self) -> Vec<String> {
        [("yt-dlp", &self.ytdlp), ("ffmpeg", &self.ffmpeg), ("deno", &self.deno)]
            .into_iter()
            .filter(|(_, path)| path.is_none())
            .map(|(name, _)| name.to_string())
            .collect()
    }

    pub fn status(&self) -> ToolsStatus {
        let missing = self.missing();
        ToolsStatus {
            ready: missing.is_empty(),
            ytdlp_version: self.ytdlp.as_ref().and_then(|_| self.ytdlp_version()),
            can_install: cfg!(windows) && find_in(&search_dirs(), "winget").is_some(),
            missing,
        }
    }

    pub fn ytdlp_version(&self) -> Option<String> {
        let output = self.run_ytdlp(["--version"]).ok()?;
        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        (!version.is_empty()).then_some(version)
    }

    /// yt-dlp is told exactly where ffmpeg and deno are, so it works even when
    /// they were installed after the app started (stale PATH).
    pub fn ytdlp_base_args(&self) -> Vec<String> {
        let mut args: Vec<String> = [
            "--ignore-config", // the user's yt-dlp config can't change our results
            "--no-playlist",   // a `&list=` link means just that one video
            "--encoding",
            "utf-8", // emoji and non-Latin titles survive the pipe
        ]
        .map(String::from)
        .to_vec();
        if let Some(ffmpeg) = &self.ffmpeg {
            args.push("--ffmpeg-location".into());
            args.push(ffmpeg.display().to_string());
        }
        if let Some(deno) = &self.deno {
            args.push("--js-runtimes".into());
            args.push(format!("deno:{}", deno.display()));
        }
        args
    }

    pub fn ytdlp_command(&self) -> Result<Command, String> {
        let path = self.ytdlp.as_ref().ok_or("yt-dlp isn't installed.")?;
        Ok(hidden_command(path))
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

    /// yt-dlp README "UPDATE": package-manager installs update through that
    /// package manager; release binaries update themselves with `-U`.
    pub fn update_command(&self) -> Result<Command, String> {
        let ytdlp = self.ytdlp.as_ref().ok_or("yt-dlp isn't installed.")?;
        if installed_with_winget(ytdlp) {
            let winget = find_in(&search_dirs(), "winget").ok_or("winget isn't available.")?;
            let mut command = hidden_command(&winget);
            command.args([
                "upgrade",
                "--id",
                "yt-dlp.yt-dlp",
                "--exact",
                "--source",
                "winget",
                "--disable-interactivity",
            ]);
            Ok(command)
        } else {
            let mut command = hidden_command(ytdlp);
            command.arg("-U");
            Ok(command)
        }
    }

    /// The winget packages that would supply what's missing. yt-dlp's package
    /// already brings ffmpeg and deno along.
    pub fn packages_to_install(&self) -> Vec<&'static str> {
        if self.ytdlp.is_none() {
            return vec!["yt-dlp.yt-dlp"];
        }
        let mut packages = Vec::new();
        if self.ffmpeg.is_none() {
            packages.push("yt-dlp.FFmpeg");
        }
        if self.deno.is_none() {
            packages.push("DenoLand.Deno");
        }
        packages
    }
}

/// Install missing tools with winget in a visible console window, so the user
/// sees what's happening and answers winget's own prompts (we never accept
/// agreements on their behalf). Blocks until winget finishes.
pub fn install_with_winget(packages: &[&str]) -> Result<(), String> {
    let winget = find_in(&search_dirs(), "winget")
        .ok_or("winget isn't available on this PC. See the README for manual install steps.")?;
    for package in packages {
        let mut command = Command::new(&winget);
        command.args(["install", "--id", package, "--exact"]);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(CREATE_NEW_CONSOLE);
        }
        command
            .status()
            .map_err(|err| format!("Couldn't start winget: {err}"))?;
    }
    Ok(())
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

/// winget-installed programs live under …\Microsoft\WinGet\…
fn installed_with_winget(path: &Path) -> bool {
    let resolved = std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    resolved.to_string_lossy().to_lowercase().contains("\\winget\\")
}

fn find_in(dirs: &[PathBuf], name: &str) -> Option<PathBuf> {
    let file_name = if cfg!(windows) { format!("{name}.exe") } else { name.to_string() };
    dirs.iter()
        .map(|dir| dir.join(&file_name))
        .find(|candidate| candidate.is_file())
}

/// Folders to search: this process's PATH, plus (on Windows) the user and
/// system PATH as currently saved in the registry and winget's link folder.
fn search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("PATH")
        .map(|path| std::env::split_paths(&path).collect())
        .unwrap_or_default();
    #[cfg(windows)]
    {
        dirs.extend(registry_path_dirs());
        if let Some(local) = std::env::var_os("LOCALAPPDATA") {
            let microsoft = PathBuf::from(local).join("Microsoft");
            dirs.push(microsoft.join("WindowsApps")); // winget's own app alias
            dirs.push(microsoft.join("WinGet").join("Links"));
        }
    }
    let mut seen = std::collections::HashSet::new();
    dirs.retain(|dir| seen.insert(dir.clone()));
    dirs
}

#[cfg(windows)]
fn registry_path_dirs() -> Vec<PathBuf> {
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    let locations = [
        (HKEY_CURRENT_USER, "Environment"),
        (
            HKEY_LOCAL_MACHINE,
            r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment",
        ),
    ];
    let mut dirs = Vec::new();
    for (hive, key) in locations {
        let value: Option<String> = RegKey::predef(hive)
            .open_subkey(key)
            .and_then(|key| key.get_value("Path"))
            .ok();
        if let Some(value) = value {
            dirs.extend(
                value
                    .split(';')
                    .filter(|part| !part.trim().is_empty())
                    .map(|part| PathBuf::from(expand_env_vars(part.trim()))),
            );
        }
    }
    dirs
}

/// Expands `%NAME%` the way Windows does for REG_EXPAND_SZ PATH entries.
#[cfg_attr(not(windows), allow(dead_code))]
fn expand_env_vars(text: &str) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(start) = rest.find('%') {
        result.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        match after.find('%') {
            Some(end) => {
                let name = &after[..end];
                match std::env::var(name) {
                    Ok(value) if !name.is_empty() => result.push_str(&value),
                    _ => {
                        result.push('%');
                        result.push_str(name);
                        result.push('%');
                    }
                }
                rest = &after[end + 1..];
            }
            None => {
                result.push_str(&rest[start..]);
                rest = "";
            }
        }
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_known_variables_and_keeps_unknown_ones() {
        std::env::set_var("VIDEO_VACUUM_TEST_DIR", r"C:\Tools");
        assert_eq!(expand_env_vars(r"%VIDEO_VACUUM_TEST_DIR%\bin"), r"C:\Tools\bin");
        assert_eq!(expand_env_vars(r"%NOT_A_REAL_VARIABLE_123%\x"), r"%NOT_A_REAL_VARIABLE_123%\x");
        assert_eq!(expand_env_vars("50% off"), "50% off");
    }

    #[test]
    fn winget_paths_are_recognized() {
        let path = Path::new(r"C:\Users\x\AppData\Local\Microsoft\WinGet\Packages\yt-dlp.yt-dlp_x\yt-dlp.exe");
        assert!(installed_with_winget(path));
        assert!(!installed_with_winget(Path::new(r"C:\Tools\yt-dlp.exe")));
    }

    #[test]
    fn yt_dlp_package_brings_everything() {
        let none = Tools::default();
        assert_eq!(none.packages_to_install(), vec!["yt-dlp.yt-dlp"]);
        let only_ytdlp = Tools { ytdlp: Some("yt-dlp".into()), ..Tools::default() };
        assert_eq!(only_ytdlp.packages_to_install(), vec!["yt-dlp.FFmpeg", "DenoLand.Deno"]);
    }
}
