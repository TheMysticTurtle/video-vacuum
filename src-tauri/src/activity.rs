//! The diagnostic log (D-31): what the app is doing, in plain words, with
//! yt-dlp's own messages attached to warnings and errors.
//!
//! - Every line is kept in memory for this session (the newest
//!   `KEEP_LINES`) and shown live in the window's Log panel.
//! - Only if the user turns on "Save logs" are lines also appended to a daily
//!   file, `activity-YYYY-MM-DD.log`, in the app's local data folder under
//!   `logs\`. Files older than `KEEP_DAYS` are deleted. Times in the files are
//!   UTC (marked "Z"); the window shows local time.
//! - Nothing here ever leaves the computer.

use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

const KEEP_LINES: usize = 2000;
pub const KEEP_DAYS: u64 = 30;
const FILE_PREFIX: &str = "activity-";
const FILE_SUFFIX: &str = ".log";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Info,
    Warn,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogLine {
    /// Milliseconds since 1970 (UTC).
    pub at_ms: u64,
    pub level: Level,
    pub text: String,
    /// yt-dlp's own words, or other technical detail; may span lines.
    pub detail: Option<String>,
}

impl LogLine {
    pub fn now(level: Level, text: impl Into<String>, detail: Option<String>) -> Self {
        let at_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        let detail = detail.map(|d| d.trim().to_string()).filter(|d| !d.is_empty());
        LogLine { at_ms, level, text: text.into(), detail }
    }
}

#[derive(Default)]
pub struct Activity {
    lines: VecDeque<LogLine>,
    /// Some(folder) while "Save logs" is on.
    save_dir: Option<PathBuf>,
}

impl Activity {
    /// Adds a line. Returns false (and adds nothing) if it repeats the line
    /// just before it exactly, apart from the time.
    pub fn push(&mut self, line: LogLine) -> bool {
        if self.lines.back().is_some_and(|last| {
            last.level == line.level && last.text == line.text && last.detail == line.detail
        }) {
            return false;
        }
        if let Some(dir) = &self.save_dir {
            let _ = append(dir, std::slice::from_ref(&line)); // a full disk must never stop a download
        }
        self.lines.push_back(line);
        while self.lines.len() > KEEP_LINES {
            self.lines.pop_front();
        }
        true
    }

    pub fn lines(&self) -> Vec<LogLine> {
        self.lines.iter().cloned().collect()
    }

    /// Empties the window's log. Saved files are kept (see `delete_saved`).
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Turns saving on (with the folder) or off. Turning it on also writes
    /// this session's lines so far, and removes files older than `KEEP_DAYS`.
    pub fn set_saving(&mut self, dir: Option<PathBuf>) -> Result<(), String> {
        if let Some(dir) = &dir {
            fs::create_dir_all(dir).map_err(|err| format!("Couldn't create {}: {err}", dir.display()))?;
            prune(dir, now_days());
            if self.save_dir.is_none() {
                let lines: Vec<LogLine> = self.lines.iter().cloned().collect();
                append(dir, &lines).map_err(|err| format!("Couldn't write the log: {err}"))?;
            }
        }
        self.save_dir = dir;
        Ok(())
    }
}

/// Appends lines to their day's file.
fn append(dir: &Path, lines: &[LogLine]) -> std::io::Result<()> {
    let mut current: Option<(String, fs::File)> = None;
    for line in lines {
        let name = file_name(line.at_ms / 1000 / 86_400);
        if current.as_ref().map(|(n, _)| n != &name).unwrap_or(true) {
            let file = OpenOptions::new().create(true).append(true).open(dir.join(&name))?;
            current = Some((name, file));
        }
        if let Some((_, file)) = current.as_mut() {
            file.write_all(format_line(line).as_bytes())?;
        }
    }
    Ok(())
}

/// One line of the saved file:
/// `2026-11-23 20:01:02Z  ERROR  text` plus indented detail lines.
pub fn format_line(line: &LogLine) -> String {
    let secs = line.at_ms / 1000;
    let (y, m, d) = civil_from_days(secs / 86_400);
    let (h, mi, s) = ((secs / 3600) % 24, (secs / 60) % 60, secs % 60);
    let level = match line.level {
        Level::Info => "INFO ",
        Level::Warn => "WARN ",
        Level::Error => "ERROR",
    };
    let mut out = format!("{y:04}-{m:02}-{d:02} {h:02}:{mi:02}:{s:02}Z  {level}  {}\r\n", line.text);
    if let Some(detail) = &line.detail {
        for detail_line in detail.lines() {
            out.push_str("        ");
            out.push_str(detail_line);
            out.push_str("\r\n");
        }
    }
    out
}

fn file_name(day: u64) -> String {
    let (y, m, d) = civil_from_days(day);
    format!("{FILE_PREFIX}{y:04}-{m:02}-{d:02}{FILE_SUFFIX}")
}

/// The day number (since 1970) in a log file's name, if it is one of ours.
fn day_of_file(name: &str) -> Option<u64> {
    let date = name.strip_prefix(FILE_PREFIX)?.strip_suffix(FILE_SUFFIX)?;
    let mut parts = date.split('-').map(|p| p.parse::<u64>().ok());
    let (y, m, d) = (parts.next()??, parts.next()??, parts.next()??);
    if parts.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    Some(days_from_civil(y, m, d))
}

/// Deletes our log files older than `KEEP_DAYS`. Other files are never touched.
pub fn prune(dir: &Path, today: u64) {
    for (path, day) in our_files(dir) {
        if day + KEEP_DAYS < today {
            let _ = fs::remove_file(path);
        }
    }
}

/// Deletes every saved log file (only ours). Returns how many were removed.
pub fn delete_saved(dir: &Path) -> Result<usize, String> {
    let mut removed = 0;
    for (path, _) in our_files(dir) {
        fs::remove_file(&path).map_err(|err| format!("Couldn't delete {}: {err}", path.display()))?;
        removed += 1;
    }
    Ok(removed)
}

fn our_files(dir: &Path) -> Vec<(PathBuf, u64)> {
    fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|entry| {
            let day = day_of_file(&entry.file_name().to_string_lossy())?;
            entry.file_type().ok()?.is_file().then(|| (entry.path(), day))
        })
        .collect()
}

fn now_days() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() / 86_400).unwrap_or(0)
}

/// Days since 1970-01-01 → (year, month, day), proleptic Gregorian.
/// Howard Hinnant's `civil_from_days` (public-domain algorithm), for days ≥ 0.
fn civil_from_days(days: u64) -> (u64, u64, u64) {
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + u64::from(m <= 2);
    (y, m, d)
}

/// The inverse of `civil_from_days`.
fn days_from_civil(y: u64, m: u64, d: u64) -> u64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y / 400;
    let yoe = y - era * 400;
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    (era * 146_097 + doe).saturating_sub(719_468)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("video-vacuum-log-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn line_at(at_ms: u64, level: Level, text: &str, detail: Option<&str>) -> LogLine {
        LogLine { at_ms, level, text: text.into(), detail: detail.map(str::to_string) }
    }

    #[test]
    fn dates_round_trip() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_415), (2025, 11, 23)); // 2025-11-23
        assert_eq!(civil_from_days(11_016), (2000, 2, 29)); // a leap day
        for day in [0, 59, 365, 11_016, 20_415, 20_780, 47_482] {
            let (y, m, d) = civil_from_days(day);
            assert_eq!(days_from_civil(y, m, d), day);
        }
    }

    #[test]
    fn lines_are_formatted_with_utc_time_and_indented_detail() {
        let line = line_at(1_763_928_062_000, Level::Error, "Download failed.", Some("ERROR: HTTP Error 403\nsecond"));
        assert_eq!(
            format_line(&line),
            "2025-11-23 20:01:02Z  ERROR  Download failed.\r\n        ERROR: HTTP Error 403\r\n        second\r\n"
        );
    }

    #[test]
    fn memory_only_until_saving_is_turned_on() {
        let dir = temp("saving");
        let mut log = Activity::default();
        log.push(line_at(1_763_928_062_000, Level::Info, "before", None));
        assert!(!dir.exists(), "nothing is written while saving is off");

        log.set_saving(Some(dir.clone())).unwrap();
        log.push(line_at(1_763_928_063_000, Level::Warn, "after", None));
        let text = fs::read_to_string(dir.join("activity-2025-11-23.log")).unwrap();
        assert!(text.contains("INFO   before") && text.contains("WARN   after"), "{text}");

        log.set_saving(None).unwrap();
        log.push(line_at(1_763_928_064_000, Level::Info, "off again", None));
        assert!(!fs::read_to_string(dir.join("activity-2025-11-23.log")).unwrap().contains("off again"));
        assert_eq!(log.lines().len(), 3);
        log.clear();
        assert!(log.lines().is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn old_files_are_pruned_and_only_our_files_are_deleted() {
        let dir = temp("prune");
        fs::create_dir_all(&dir).unwrap();
        let today = days_from_civil(2025, 11, 23);
        for name in ["activity-2025-11-23.log", "activity-2025-10-24.log", "activity-2025-10-23.log", "notes.txt", "activity-bad.log"] {
            fs::write(dir.join(name), "x").unwrap();
        }
        prune(&dir, today);
        let mut left: Vec<String> =
            fs::read_dir(&dir).unwrap().flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        // 30 days back (10-24) is kept; 31 days back (10-23) is removed; foreign files stay.
        assert_eq!(left, ["activity-2025-10-24.log", "activity-2025-11-23.log", "activity-bad.log", "notes.txt"]);

        assert_eq!(delete_saved(&dir).unwrap(), 2);
        assert!(dir.join("notes.txt").exists() && dir.join("activity-bad.log").exists());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn the_log_keeps_only_the_newest_lines() {
        let mut log = Activity::default();
        assert!(log.push(line_at(0, Level::Info, "same", None)));
        assert!(!log.push(line_at(9, Level::Info, "same", None)), "an exact repeat is skipped");
        assert!(log.push(line_at(9, Level::Warn, "same", None)), "a different level is not a repeat");
        log.clear();
        for i in 0..(KEEP_LINES as u64 + 5) {
            log.push(line_at(i, Level::Info, &i.to_string(), None));
        }
        let lines = log.lines();
        assert_eq!(lines.len(), KEEP_LINES);
        assert_eq!(lines[0].text, "5");
    }
}
