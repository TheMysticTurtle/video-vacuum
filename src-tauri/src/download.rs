//! Running one download: picking a file name that's free, starting yt-dlp,
//! reporting progress, cancelling, and cleaning up after a cancel or failure.

use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::link::clean_youtube_url;
use crate::store::HistoryEntry;
use crate::tools::{hidden_command, Tools};
use crate::ytdlp::{
    escape_output_template, format_args, friendly_error, parse_line, progress_args, sanitize_file_name, Format, Line,
};

/// What the page asks for.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    pub url: String,
    pub format: Format,
    /// None = best available.
    pub height: Option<u32>,
    pub folder: String,
    /// Without extension; made safe and unique here.
    pub file_name: String,
    // Shown in the history:
    pub title: String,
    pub channel: String,
    pub thumbnail: Option<String>,
}

/// Messages sent to the page while a download runs (a Tauri Channel).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum DownloadEvent {
    Started { file_name: String },
    Progress { stage: &'static str, percent: f64, speed: Option<f64>, eta: Option<u64> },
    Finishing,
    Done { entry: HistoryEntry },
    Failed { message: String, detail: String },
    Cancelled,
}

/// Shared between the download thread and the Cancel button.
#[derive(Default)]
pub struct JobControl {
    pid: AtomicU32,
    cancelled: AtomicBool,
}

impl JobControl {
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        let pid = self.pid.load(Ordering::SeqCst);
        if pid != 0 {
            kill_tree(pid);
        }
    }
}

/// A checked request, ready to run.
#[derive(Debug)]
pub struct Prepared {
    pub url: String,
    pub folder: PathBuf,
    pub stem: String,
    /// Files starting with "stem." that were already there (lowercase), so a
    /// cleanup never touches them.
    existing: HashSet<String>,
}

pub fn prepare(request: &DownloadRequest) -> Result<Prepared, String> {
    let url = clean_youtube_url(&request.url).ok_or("That doesn't look like a YouTube video link.")?;
    let folder = PathBuf::from(request.folder.trim());
    if request.folder.trim().is_empty() || !folder.is_absolute() {
        return Err("Choose a folder to save into.".into());
    }
    fs::create_dir_all(&folder).map_err(|err| format!("Couldn't use {}: {err}", folder.display()))?;
    let stem = unique_stem(&folder, &sanitize_file_name(&request.file_name), request.format.extensions());
    let existing = files_with_stem(&folder, &stem);
    Ok(Prepared { url, folder, stem, existing })
}

/// Never overwrite: "name", then "name (2)", "name (3)"… A name is taken when
/// a file of the same type (or a download still in progress) already uses it,
/// so "Clip.mp4" and "Clip.m4a" can sit side by side.
fn unique_stem(folder: &Path, base: &str, extensions: &[&str]) -> String {
    (1..)
        .map(|n| if n == 1 { base.to_string() } else { format!("{base} ({n})") })
        .find(|stem| !stem_taken(folder, stem, extensions))
        .expect("an unused name always exists")
}

fn stem_taken(folder: &Path, stem: &str, extensions: &[&str]) -> bool {
    extensions.iter().any(|ext| folder.join(format!("{stem}.{ext}")).exists())
        || files_with_stem(folder, stem)
            .iter()
            .any(|name| name.ends_with(".part") || name.ends_with(".ytdl"))
}

/// Lowercase names of the files in `folder` that start with "stem.".
fn files_with_stem(folder: &Path, stem: &str) -> HashSet<String> {
    let prefix = format!("{}.", stem.to_lowercase());
    fs::read_dir(folder)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().to_lowercase())
                .filter(|name| name.starts_with(&prefix))
                .collect()
        })
        .unwrap_or_default()
}

/// After a cancel or failure, remove what this download created (partial
/// `.part` / `.ytdl` / `.f137.mp4` files), and nothing that was there before.
fn clean_up(prepared: &Prepared) {
    for name in files_with_stem(&prepared.folder, &prepared.stem) {
        if !prepared.existing.contains(&name) {
            let _ = fs::remove_file(prepared.folder.join(&name));
        }
    }
}

/// Run the download to the end, sending events as it goes. Returns the
/// finished file's history entry, or None if it failed or was cancelled.
pub fn run(
    tools: &Tools,
    request: &DownloadRequest,
    prepared: &Prepared,
    job: &JobControl,
    send: &dyn Fn(DownloadEvent),
) -> Option<HistoryEntry> {
    send(DownloadEvent::Started { file_name: prepared.stem.clone() });

    let mut command = match tools.ytdlp_command() {
        Ok(command) => command,
        Err(message) => {
            send(DownloadEvent::Failed { message, detail: String::new() });
            return None;
        }
    };
    command
        .args(tools.ytdlp_base_args())
        .args(progress_args())
        .args(format_args(request.format, request.height))
        .arg("--no-overwrites")
        .arg("-P")
        .arg(&prepared.folder)
        .arg("-o")
        .arg(format!("{}.%(ext)s", escape_output_template(&prepared.stem)))
        .arg("--")
        .arg(&prepared.url)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            send(DownloadEvent::Failed { message: "Couldn't start yt-dlp.".into(), detail: err.to_string() });
            return None;
        }
    };
    job.pid.store(child.id(), Ordering::SeqCst);
    if job.cancelled.load(Ordering::SeqCst) {
        kill_tree(child.id()); // Cancel was pressed before the process existed
    }

    // Read stdout and stderr together, line by line.
    let (lines_tx, lines_rx) = mpsc::channel::<String>();
    let readers: Vec<_> = [
        child.stdout.take().map(|s| Box::new(s) as Box<dyn Read + Send>),
        child.stderr.take().map(|s| Box::new(s) as Box<dyn Read + Send>),
    ]
    .into_iter()
    .flatten()
    .map(|stream| {
        let tx = lines_tx.clone();
        thread::spawn(move || {
            for line in BufReader::new(stream).split(b'\n').map_while(Result::ok) {
                if tx.send(String::from_utf8_lossy(&line).into_owned()).is_err() {
                    break;
                }
            }
        })
    })
    .collect();
    drop(lines_tx);

    let mut final_path: Option<String> = None;
    let mut other_output = String::new();
    for line in lines_rx {
        match parse_line(&line) {
            Line::Progress { stage, percent, speed, eta } => {
                send(DownloadEvent::Progress { stage, percent, speed, eta })
            }
            Line::Finishing => send(DownloadEvent::Finishing),
            Line::File(path) => final_path = Some(path),
            Line::Other(text) => {
                other_output.push_str(&text);
                other_output.push('\n');
            }
        }
    }
    for reader in readers {
        let _ = reader.join();
    }
    let _ = child.wait();

    if job.cancelled.load(Ordering::SeqCst) {
        clean_up(prepared);
        send(DownloadEvent::Cancelled);
        return None;
    }
    // `after_move` is printed only once the file is complete and in place.
    match final_path.filter(|path| Path::new(path).is_file()) {
        Some(path) => {
            let entry = HistoryEntry {
                id: new_id(),
                title: request.title.clone(),
                video_url: prepared.url.clone(),
                channel: request.channel.clone(),
                thumbnail: request.thumbnail.clone(),
                format: request.format.label().into(),
                quality: if request.format.is_audio_only() {
                    "Audio".into()
                } else {
                    request.height.map_or_else(|| "Best available".into(), |h| format!("{h}p"))
                },
                size_bytes: fs::metadata(&path).ok().map(|m| m.len()),
                file_path: path,
                downloaded_at: now_seconds(),
            };
            Some(entry)
        }
        None => {
            clean_up(prepared);
            let error = friendly_error(&other_output);
            send(DownloadEvent::Failed { message: error.message, detail: error.detail });
            None
        }
    }
}

/// Stop yt-dlp *and* the ffmpeg it started. On Windows, killing only the
/// parent would leave ffmpeg running, so the whole tree is ended.
fn kill_tree(pid: u32) {
    #[cfg(windows)]
    let mut command = {
        let mut command = hidden_command(Path::new("taskkill"));
        command.args(["/PID", &pid.to_string(), "/T", "/F"]);
        command
    };
    #[cfg(not(windows))]
    let mut command = {
        let mut command = hidden_command(Path::new("kill"));
        command.args(["-TERM", &pid.to_string()]);
        command
    };
    let _ = command.stdout(Stdio::null()).stderr(Stdio::null()).status();
}

pub fn now_seconds() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

pub fn new_id() -> String {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos());
    format!("{nanos:x}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_are_never_reused_and_cleanup_only_touches_ours() {
        let dir = std::env::temp_dir().join(format!("video-vacuum-names-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        let mp4 = Format::Mp4.extensions();
        assert_eq!(unique_stem(&dir, "Clip", mp4), "Clip");
        fs::write(dir.join("Clip.txt"), b"x").unwrap(); // not the same type: name still free
        fs::write(dir.join("Clip.m4a"), b"x").unwrap();
        assert_eq!(unique_stem(&dir, "Clip", mp4), "Clip");
        fs::write(dir.join("Clip.mp4"), b"x").unwrap();
        assert_eq!(unique_stem(&dir, "Clip", mp4), "Clip (2)");
        fs::write(dir.join("clip (2).MP4"), b"x").unwrap(); // case-insensitive
        assert_eq!(unique_stem(&dir, "Clip", mp4), "Clip (3)");
        fs::write(dir.join("Clip (3).webm.part"), b"x").unwrap(); // another download in progress
        assert_eq!(unique_stem(&dir, "Clip", mp4), "Clip (4)");
        fs::write(dir.join("Clip.mkv"), b"x").unwrap(); // WebM may end up as .mkv
        assert_eq!(unique_stem(&dir, "Clip", Format::Webm.extensions()), "Clip (2)");

        // A download into "Notes" where "Notes.txt" already exists: cleanup keeps Notes.txt.
        fs::write(dir.join("Notes.txt"), b"x").unwrap();
        let prepared = Prepared {
            url: String::new(),
            folder: dir.clone(),
            stem: "Notes".into(),
            existing: files_with_stem(&dir, "Notes"),
        };
        fs::write(dir.join("Notes.f137.mp4.part"), b"x").unwrap();
        fs::write(dir.join("Notes.ytdl"), b"x").unwrap();
        fs::write(dir.join("Notes (2).mp4"), b"x").unwrap(); // a different name: must survive
        clean_up(&prepared);
        let mut left: Vec<String> =
            fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        assert_eq!(
            left,
            [
                "Clip (3).webm.part",
                "Clip.m4a",
                "Clip.mkv",
                "Clip.mp4",
                "Clip.txt",
                "Notes (2).mp4",
                "Notes.txt",
                "clip (2).MP4"
            ]
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn requests_are_checked() {
        let request = DownloadRequest {
            url: "https://youtu.be/jNQXAC9IVRw?si=x".into(),
            format: Format::Mp4,
            height: None,
            folder: "relative/folder".into(),
            file_name: "x".into(),
            title: "t".into(),
            channel: "c".into(),
            thumbnail: None,
        };
        assert!(prepare(&request).is_err()); // folder must be absolute
        let bad_link = DownloadRequest { url: "https://example.com".into(), ..request };
        assert!(prepare(&bad_link).unwrap_err().contains("YouTube"));
    }
}
