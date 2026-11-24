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


use crate::activity::Level;
use crate::store::HistoryEntry;
use crate::tools::{hidden_command, Tools};
use crate::ytdlp::{
    escape_output_template, format_args, friendly_error, parse_line, progress_args, sanitize_file_name, step_words,
    Format, Line, Lookup, VideoInfo, MP3_BITRATES, MP3_DEFAULT_KBPS,
};

/// What the page asks for: only the choices. The video itself (link, title,
/// thumbnail…) comes from the lookup the app already made, so nothing makes a
/// round trip through the page.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadRequest {
    /// The looked-up video this is for (guards against a newer lookup in between).
    pub video_id: String,
    pub format: Format,
    /// None = best available.
    pub height: Option<u32>,
    /// MP3 only: kbit/s, one of `MP3_BITRATES`. None = the default.
    #[serde(default)]
    pub mp3_kbps: Option<u32>,
    pub folder: String,
    /// Without extension; made safe and unique here.
    pub file_name: String,
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

/// Where yt-dlp gets the video from.
#[derive(Debug)]
enum Source {
    /// The saved lookup (`--load-info-json`): no second trip to YouTube.
    /// Measured 2026-11-22: 1.75 s instead of 3.9 s for a short video.
    InfoJson(PathBuf),
    /// A fresh lookup from the link, when the saved one is too old to trust.
    Url(String),
}

/// A checked request, ready to run.
#[derive(Debug)]
pub struct Prepared {
    pub video: VideoInfo,
    source: Source,
    pub folder: PathBuf,
    pub stem: String,
    /// Files starting with "stem." that were already there (lowercase), so a
    /// cleanup never touches them.
    existing: HashSet<String>,
}

impl Drop for Prepared {
    /// The saved-lookup file is only needed for this one download.
    fn drop(&mut self) {
        if let Source::InfoJson(path) = &self.source {
            let _ = fs::remove_file(path);
        }
    }
}

/// `lookup` is the app's latest lookup; `temp_dir` is where its JSON is written for yt-dlp.
pub fn prepare(
    request: &DownloadRequest,
    lookup: Option<&Lookup>,
    temp_dir: &Path,
    job_id: &str,
) -> Result<Prepared, String> {
    let lookup = lookup
        .filter(|lookup| lookup.info.id == request.video_id)
        .ok_or("Look up the video again, then download.")?;
    if request.format == Format::Mp3 && !MP3_BITRATES.contains(&mp3_kbps(request)) {
        return Err("Choose an MP3 bitrate from the list.".into());
    }
    let folder = PathBuf::from(request.folder.trim());
    if request.folder.trim().is_empty() || !folder.is_absolute() {
        return Err("Choose a folder to save into.".into());
    }
    fs::create_dir_all(&folder).map_err(|err| format!("Couldn't use {}: {err}", folder.display()))?;

    // Read the folder once; pick the name and take the cleanup snapshot from that.
    let names = folder_names(&folder);
    let stem = unique_stem(&names, &sanitize_file_name(&request.file_name), request.format.collides_with());
    let existing = with_stem(&names, &stem);

    let source = if lookup.is_fresh() {
        let path = temp_dir.join(format!("lookup-{job_id}.json"));
        match fs::create_dir_all(temp_dir).and_then(|()| fs::write(&path, &lookup.info_json)) {
            Ok(()) => Source::InfoJson(path),
            Err(_) => Source::Url(lookup.info.url.clone()), // still works, just slower
        }
    } else {
        Source::Url(lookup.info.url.clone())
    };
    Ok(Prepared { video: lookup.info.clone(), source, folder, stem, existing })
}

/// "1080p", "Best available", "Audio", "192 kbps": for history and the log.
fn quality_label(request: &DownloadRequest) -> String {
    if request.format == Format::Mp3 {
        format!("{} kbps", mp3_kbps(request))
    } else if request.format.is_audio_only() {
        "Audio".into()
    } else {
        request.height.map_or_else(|| "Best available".into(), |h| format!("{h}p"))
    }
}

fn mp3_kbps(request: &DownloadRequest) -> u32 {
    request.mp3_kbps.unwrap_or(MP3_DEFAULT_KBPS)
}

/// Lowercase names of everything in `folder`.
fn folder_names(folder: &Path) -> HashSet<String> {
    fs::read_dir(folder)
        .map(|entries| entries.flatten().map(|e| e.file_name().to_string_lossy().to_lowercase()).collect())
        .unwrap_or_default()
}

/// The names that start with "stem." (`names` are lowercase).
fn with_stem(names: &HashSet<String>, stem: &str) -> HashSet<String> {
    let prefix = format!("{}.", stem.to_lowercase());
    names.iter().filter(|name| name.starts_with(&prefix)).cloned().collect()
}

/// Never overwrite: "name", then "name (2)", "name (3)"… A name is taken when
/// a file of the same type (or a download still in progress) already uses it,
/// so "Clip.mp4" and "Clip.m4a" can sit side by side.
fn unique_stem(names: &HashSet<String>, base: &str, extensions: &[&str]) -> String {
    (1..)
        .map(|n| if n == 1 { base.to_string() } else { format!("{base} ({n})") })
        .find(|stem| !stem_taken(names, stem, extensions))
        .expect("an unused name always exists")
}

fn stem_taken(names: &HashSet<String>, stem: &str, extensions: &[&str]) -> bool {
    let lower = stem.to_lowercase();
    extensions.iter().any(|ext| names.contains(&format!("{lower}.{ext}")))
        || with_stem(names, stem).iter().any(|name| name.ends_with(".part") || name.ends_with(".ytdl"))
}

/// After a cancel or failure, remove what this download created (partial
/// `.part` / `.ytdl` / `.f137.mp4` files), and nothing that was there before.
fn clean_up(folder: &Path, stem: &str, existing: &HashSet<String>) {
    for name in with_stem(&folder_names(folder), stem) {
        if !existing.contains(&name) {
            let _ = fs::remove_file(folder.join(&name));
        }
    }
}

/// What the diagnostic log is told (D-31): a level, plain words, optional detail.
pub type Note<'a> = &'a dyn Fn(Level, String, Option<String>);

/// Run the download to the end, sending events to the window and plain-word
/// steps to the diagnostic log as it goes. Returns the finished file's history
/// entry, or None if it failed or was cancelled.
pub fn run(
    tools: &Tools,
    request: &DownloadRequest,
    prepared: &Prepared,
    job: &JobControl,
    send: &dyn Fn(DownloadEvent),
    note: Note,
) -> Option<HistoryEntry> {
    send(DownloadEvent::Started { file_name: prepared.stem.clone() });
    let source = match &prepared.source {
        Source::InfoJson(_) => "the lookup just made (no second trip to YouTube)",
        Source::Url(_) => "the link (the earlier lookup was over an hour old)",
    };
    note(
        Level::Info,
        format!("Downloading \"{}\" as {} ({})", prepared.video.title, request.format.label(), quality_label(request)),
        Some(format!("Folder: {}\nFrom: {source}", prepared.folder.display())),
    );

    let mut command = match tools.ytdlp_command() {
        Ok(command) => command,
        Err(message) => {
            note(Level::Error, message.clone(), None);
            send(DownloadEvent::Failed { message, detail: String::new() });
            return None;
        }
    };
    command
        .args(tools.ytdlp_base_args())
        .args(progress_args())
        .args(format_args(request.format, request.height, mp3_kbps(request)))
        .arg("--no-overwrites")
        .arg("-P")
        .arg(&prepared.folder)
        .arg("-o")
        .arg(format!("{}.%(ext)s", escape_output_template(&prepared.stem)));
    match &prepared.source {
        Source::InfoJson(path) => command.arg("--load-info-json").arg(path),
        Source::Url(url) => command.arg("--").arg(url),
    };
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(err) => {
            note(Level::Error, "Couldn't start yt-dlp.".into(), Some(err.to_string()));
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
    // Log each stream once when it starts, and each finishing step once.
    let mut stage_logged: Option<&'static str> = None;
    let mut step_logged = String::new();
    for line in lines_rx {
        match parse_line(&line) {
            Line::Progress { stage, percent, speed, eta } => {
                if stage_logged != Some(stage) {
                    stage_logged = Some(stage);
                    note(Level::Info, format!("Downloading the {stage} stream"), None);
                }
                send(DownloadEvent::Progress { stage, percent, speed, eta })
            }
            Line::Finishing(step) => {
                if step != step_logged {
                    note(Level::Info, step_words(&step, request.format), None);
                    step_logged = step;
                }
                send(DownloadEvent::Finishing)
            }
            Line::File(path) => final_path = Some(path),
            Line::Other(text) => {
                if let Some(warning) = text.trim().strip_prefix("WARNING:") {
                    note(Level::Warn, "yt-dlp warning".into(), Some(warning.trim().to_string()));
                }
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
        clean_up(&prepared.folder, &prepared.stem, &prepared.existing);
        note(Level::Info, "Cancelled. The unfinished files were removed.".into(), None);
        send(DownloadEvent::Cancelled);
        return None;
    }
    // `after_move` is printed only once the file is complete and in place.
    match final_path.filter(|path| Path::new(path).is_file()) {
        Some(path) => {
            let entry = HistoryEntry {
                id: new_id(),
                title: prepared.video.title.clone(),
                video_url: prepared.video.url.clone(),
                channel: prepared.video.channel.clone(),
                thumbnail: prepared.video.thumbnail.clone(),
                format: request.format.label().into(),
                quality: quality_label(request),
                size_bytes: fs::metadata(&path).ok().map(|m| m.len()),
                file_path: path,
                downloaded_at: now_seconds(),
            };
            let size = entry.size_bytes.map(|b| format!(" ({:.1} MB)", b as f64 / 1e6)).unwrap_or_default();
            note(Level::Info, format!("Saved {}{size}", entry.file_path), None);
            Some(entry)
        }
        None => {
            clean_up(&prepared.folder, &prepared.stem, &prepared.existing);
            let error = friendly_error(&other_output);
            // The log gets yt-dlp's last lines, not just the one shown in the window.
            let lines: Vec<&str> = other_output.lines().filter(|l| !l.trim().is_empty()).collect();
            let tail = lines[lines.len().saturating_sub(15)..].join("\n");
            note(Level::Error, format!("Download failed: {}", error.message), Some(tail));
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
    use std::time::{Duration, SystemTime};

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("video-vacuum-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn lookup(id: &str, age: Duration) -> Lookup {
        Lookup {
            info: VideoInfo {
                url: format!("https://www.youtube.com/watch?v={id}"),
                id: id.into(),
                title: "Me at the zoo".into(),
                channel: "jawed".into(),
                channel_url: None,
                thumbnail: Some("https://i.ytimg.com/vi/x/hqdefault.jpg".into()),
                duration_seconds: Some(19.0),
                upload_date: None,
                view_count: None,
                description: String::new(),
                heights: vec![240],
                suggested_file_name: "Me at the zoo".into(),
            },
            info_json: br#"{"id": "jNQXAC9IVRw"}"#.to_vec(),
            fetched: SystemTime::now() - age,
        }
    }

    /// D-33: a download must never pick a name whose in-between file would
    /// be an existing file of another type (yt-dlp would use and delete it).
    #[test]
    fn audio_downloads_never_reuse_a_name_held_by_another_media_file() {
        let dir = temp("collide");
        let taken = |base: &str, format: Format| unique_stem(&folder_names(&dir), base, format.collides_with());
        fs::write(dir.join("Zoo.webm"), b"video").unwrap();
        assert_eq!(taken("Zoo", Format::Mp3), "Zoo (2)", "MP3 audio arrives as Zoo.webm");
        assert_eq!(taken("Zoo", Format::M4a), "Zoo (2)", "M4A may fall back to a .webm source");
        assert_eq!(taken("Zoo", Format::Mkv), "Zoo (2)", "MKV may repackage a .webm");
        assert_eq!(taken("Zoo", Format::Mp4), "Zoo (2)", "MP4 may repackage a .webm");
        fs::write(dir.join("Song.m4a"), b"audio").unwrap();
        assert_eq!(taken("Song", Format::Mp3), "Song (2)", "MP3 audio may arrive as Song.m4a");
        assert_eq!(taken("Song", Format::Webm), "Song", "a WebM download never makes Song.m4a");
        fs::write(dir.join("Talk.mp3"), b"audio").unwrap();
        assert_eq!(taken("Talk", Format::Mp4), "Talk", "an MP4 download never makes Talk.mp3");
        for format in [Format::Mp4, Format::Webm, Format::Mkv, Format::M4a, Format::Mp3] {
            let own = format.label().to_lowercase();
            let own = if own == "webm" { "webm".to_string() } else { own };
            assert!(format.collides_with().contains(&own.as_str()), "{own}: its own type always counts");
        }
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn names_are_never_reused_and_cleanup_only_touches_ours() {
        let dir = temp("names");
        let taken = |base: &str, exts: &[&str]| unique_stem(&folder_names(&dir), base, exts);

        let mp4 = Format::Mp4.collides_with();
        assert_eq!(taken("Clip", mp4), "Clip");
        fs::write(dir.join("Clip.txt"), b"x").unwrap(); // can't clash: name still free
        fs::write(dir.join("Clip.m4a"), b"x").unwrap(); // an MP4 download never makes Clip.m4a
        assert_eq!(taken("Clip", mp4), "Clip");
        fs::write(dir.join("Clip.mp4"), b"x").unwrap();
        assert_eq!(taken("Clip", mp4), "Clip (2)");
        fs::write(dir.join("clip (2).MP4"), b"x").unwrap(); // case-insensitive
        assert_eq!(taken("Clip", mp4), "Clip (3)");
        fs::write(dir.join("Clip (3).webm.part"), b"x").unwrap(); // another download in progress
        assert_eq!(taken("Clip", mp4), "Clip (4)");
        fs::write(dir.join("Clip.mkv"), b"x").unwrap(); // WebM may end up as .mkv
        assert_eq!(taken("Clip", Format::Webm.collides_with()), "Clip (2)");

        // A download into "Notes" where "Notes.txt" already exists: cleanup keeps Notes.txt.
        fs::write(dir.join("Notes.txt"), b"x").unwrap();
        let existing = with_stem(&folder_names(&dir), "Notes");
        fs::write(dir.join("Notes.f137.mp4.part"), b"x").unwrap();
        fs::write(dir.join("Notes.ytdl"), b"x").unwrap();
        fs::write(dir.join("Notes (2).mp4"), b"x").unwrap(); // a different name: must survive
        clean_up(&dir, "Notes", &existing);
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
        let dir = temp("requests");
        let request = DownloadRequest {
            video_id: "jNQXAC9IVRw".into(),
            format: Format::Mp4,
            height: None,
            mp3_kbps: None,
            folder: dir.display().to_string(),
            file_name: "x".into(),
        };
        let fresh = lookup("jNQXAC9IVRw", Duration::ZERO);
        assert!(prepare(&request, None, &dir, "j1").unwrap_err().contains("Look up"));
        let other = lookup("aqz-KE-bpKQ", Duration::ZERO);
        assert!(prepare(&request, Some(&other), &dir, "j2").unwrap_err().contains("Look up"));
        let relative = DownloadRequest { folder: "relative/folder".into(), ..request.clone() };
        assert!(prepare(&relative, Some(&fresh), &dir, "j3").is_err());
        let odd_bitrate = DownloadRequest { format: Format::Mp3, mp3_kbps: Some(999), ..request.clone() };
        assert!(prepare(&odd_bitrate, Some(&fresh), &dir, "j4").unwrap_err().contains("bitrate"));
        let mp3 = DownloadRequest { format: Format::Mp3, mp3_kbps: Some(320), ..request.clone() };
        assert!(prepare(&mp3, Some(&fresh), &dir, "j5").is_ok());
        let default_mp3 = DownloadRequest { format: Format::Mp3, mp3_kbps: None, ..request.clone() };
        assert!(prepare(&default_mp3, Some(&fresh), &dir, "j6").is_ok());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn fresh_lookups_are_reused_and_cleaned_up_stale_ones_are_not() {
        let dir = temp("source");
        let request = DownloadRequest {
            video_id: "jNQXAC9IVRw".into(),
            format: Format::Mp4,
            height: None,
            mp3_kbps: None,
            folder: dir.display().to_string(),
            file_name: "x".into(),
        };
        let prepared = prepare(&request, Some(&lookup("jNQXAC9IVRw", Duration::ZERO)), &dir, "j1").unwrap();
        let json_file = dir.join("lookup-j1.json");
        assert!(matches!(&prepared.source, Source::InfoJson(path) if *path == json_file));
        assert_eq!(fs::read(&json_file).unwrap(), br#"{"id": "jNQXAC9IVRw"}"#);
        assert_eq!(prepared.video.title, "Me at the zoo"); // history comes from the lookup
        drop(prepared);
        assert!(!json_file.exists(), "the saved lookup is removed after the download");

        let old = lookup("jNQXAC9IVRw", Duration::from_secs(2 * 60 * 60));
        let prepared = prepare(&request, Some(&old), &dir, "j2").unwrap();
        assert!(matches!(&prepared.source, Source::Url(url) if url.ends_with("jNQXAC9IVRw")));
        drop(prepared);
        let _ = fs::remove_dir_all(dir);
    }
}
