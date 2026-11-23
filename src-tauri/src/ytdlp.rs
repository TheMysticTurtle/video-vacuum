//! Everything about talking to yt-dlp: reading video info, choosing formats,
//! reading progress lines and turning errors into plain words.
//!
//! References: yt-dlp README ("Sorting Formats", "OUTPUT TEMPLATE") and the
//! vault note "yt-dlp - Using It From Your Own App".

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{Duration, SystemTime};

use crate::tools::Tools;

/// What the page shows about a video.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInfo {
    pub url: String,
    pub id: String,
    pub title: String,
    pub channel: String,
    pub channel_url: Option<String>,
    pub thumbnail: Option<String>,
    pub duration_seconds: Option<f64>,
    /// `YYYY-MM-DD`
    pub upload_date: Option<String>,
    pub view_count: Option<u64>,
    pub description: String,
    /// Every resolution on offer, highest first (shorter side, like `-S res`).
    pub heights: Vec<u32>,
    /// The title, made safe to use as a file name.
    pub suggested_file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    Mp4,
    Webm,
    Mkv,
    M4a,
    /// The one format that's converted (D-27): YouTube sends no MP3.
    Mp3,
}

/// The MP3 bitrates on offer, in kbit/s. YouTube's own audio is about
/// 130-160 kbit/s, so 192 is the sensible default; 320 only makes the file bigger.
pub const MP3_BITRATES: [u32; 4] = [128, 192, 256, 320];
pub const MP3_DEFAULT_KBPS: u32 = 192;

impl Format {
    pub fn label(self) -> &'static str {
        match self {
            Format::Mp4 => "MP4",
            Format::Webm => "WebM",
            Format::Mkv => "MKV",
            Format::M4a => "M4A",
            Format::Mp3 => "MP3",
        }
    }

    pub fn is_audio_only(self) -> bool {
        matches!(self, Format::M4a | Format::Mp3)
    }

    /// The extensions the finished file can end up with.
    pub fn extensions(self) -> &'static [&'static str] {
        match self {
            Format::Mp4 => &["mp4"],
            Format::Webm => &["webm", "mkv"], // falls back to .mkv
            Format::Mkv => &["mkv"],
            Format::M4a => &["m4a"],
            Format::Mp3 => &["mp3"],
        }
    }
}

/// A finished lookup: what the page shows, plus yt-dlp's full JSON so the
/// download can reuse it (`--load-info-json`) instead of asking YouTube again.
pub struct Lookup {
    pub info: VideoInfo,
    pub info_json: Vec<u8>,
    pub fetched: SystemTime,
}

/// YouTube's stream links expire after a few hours; reuse a lookup only while
/// it's well inside that.
pub const LOOKUP_REUSE_LIMIT: Duration = Duration::from_secs(60 * 60);

impl Lookup {
    pub fn is_fresh(&self) -> bool {
        self.fetched.elapsed().is_ok_and(|age| age < LOOKUP_REUSE_LIMIT) // clock went backwards = not fresh
    }
}

/// Ask yt-dlp about a video. `-J` prints its info as JSON and downloads nothing.
pub fn video_info(tools: &Tools, url: &str) -> Result<Lookup, String> {
    let mut args = tools.ytdlp_base_args();
    args.extend(["-J".into(), "--".into(), url.into()]);
    let output = tools.run_ytdlp(&args)?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(friendly_error(&stderr).message);
    }
    let parsed: Value = serde_json::from_slice(&output.stdout)
        .map_err(|err| format!("yt-dlp returned something unexpected: {err}"))?;
    Ok(Lookup { info: parse_info(url, &parsed), info_json: output.stdout, fetched: SystemTime::now() })
}

/// The smallest thumbnail at least 400 px wide (sharp at 200 px on high-DPI
/// screens). yt-dlp's own `thumbnail` is the biggest one, often a 1920×1080
/// WebP of ~130 KB; the 480 px one is ~30 KB.
fn pick_thumbnail(info: &Value) -> Option<String> {
    const WANTED_WIDTH: u64 = 400;
    let sized = info
        .get("thumbnails")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|t| Some((t.get("width")?.as_u64()?, t.get("url")?.as_str()?)))
        .filter(|(width, url)| *width >= WANTED_WIDTH && url.starts_with("https://"))
        .min_by_key(|(width, _)| *width)
        .map(|(_, url)| url.to_string());
    sized.or_else(|| info.get("thumbnail").and_then(Value::as_str).map(str::to_string))
}

fn parse_info(url: &str, info: &Value) -> VideoInfo {
    let text = |key: &str| info.get(key).and_then(Value::as_str).map(str::to_string);
    let title = text("title").unwrap_or_else(|| "Untitled".into());

    let mut heights: Vec<u32> = info
        .get("formats")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|f| !matches!(f.get("vcodec").and_then(Value::as_str), None | Some("none")))
        .filter_map(|f| {
            let height = f.get("height").and_then(Value::as_u64)?;
            let width = f.get("width").and_then(Value::as_u64).unwrap_or(height);
            u32::try_from(width.min(height)).ok()
        })
        .collect();
    heights.sort_unstable_by(|a, b| b.cmp(a));
    heights.dedup();

    VideoInfo {
        url: url.to_string(),
        id: text("id").unwrap_or_default(),
        channel: text("channel")
            .or_else(|| text("uploader"))
            .unwrap_or_else(|| "Unknown".into()),
        channel_url: text("channel_url").or_else(|| text("uploader_url")),
        thumbnail: pick_thumbnail(info),
        duration_seconds: info.get("duration").and_then(Value::as_f64),
        upload_date: text("upload_date").and_then(|d| format_upload_date(&d)),
        view_count: info.get("view_count").and_then(Value::as_u64),
        description: text("description").unwrap_or_default(),
        heights,
        suggested_file_name: sanitize_file_name(&title),
        title,
    }
}

/// `20050423` → `2005-04-23`
fn format_upload_date(raw: &str) -> Option<String> {
    (raw.len() == 8 && raw.bytes().all(|b| b.is_ascii_digit()))
        .then(|| format!("{}-{}-{}", &raw[..4], &raw[4..6], &raw[6..]))
}

/// yt-dlp options for each format. YouTube sends video and audio separately;
/// yt-dlp picks the best pair and ffmpeg joins them without re-encoding.
/// `-S res:720` = best quality no larger than 720p (README, "Sorting Formats").
/// MP3 is the exception: ffmpeg converts the best audio at `mp3_kbps`
/// (README, "Post-Processing Options": `--audio-format`, `--audio-quality`).
///
/// A future clip/section feature adds its `--download-sections` options next
/// to these; nothing else in the download path needs to change.
pub fn format_args(format: Format, height: Option<u32>, mp3_kbps: u32) -> Vec<String> {
    let res = height.map_or_else(|| "res".to_string(), |h| format!("res:{h}"));
    let args: Vec<String> = match format {
        // Most compatible: H.264 + AAC. YouTube only offers H.264 up to 1080p;
        // above that it's VP9/AV1, still in an .mp4.
        Format::Mp4 => vec![
            "-S".into(),
            format!("{res},codec:h264:m4a"),
            "--merge-output-format".into(),
            "mp4".into(),
            "--remux-video".into(),
            "mp4".into(),
        ],
        // VP9/AV1 + Opus; falls back to .mkv if no WebM pair is offered.
        Format::Webm => vec![
            "-S".into(),
            format!("{res},ext:webm:webm"),
            "--merge-output-format".into(),
            "webm/mkv".into(),
        ],
        // Holds anything: the very best streams, kept as-is.
        Format::Mkv => vec![
            "-S".into(),
            res,
            "--merge-output-format".into(),
            "mkv".into(),
            "--remux-video".into(),
            "mkv".into(),
        ],
        // Audio only. YouTube's AAC audio already is .m4a, so nothing is converted.
        Format::M4a => vec![
            "-f".into(),
            "ba[ext=m4a]/ba/b".into(),
            "-x".into(),
            "--audio-format".into(),
            "m4a".into(),
        ],
        // Audio only, converted: the best audio (usually Opus), re-encoded by
        // ffmpeg's LAME encoder at a constant bitrate.
        Format::Mp3 => vec![
            "-f".into(),
            "ba/b".into(),
            "-x".into(),
            "--audio-format".into(),
            "mp3".into(),
            "--audio-quality".into(),
            format!("{mp3_kbps}K"),
        ],
    };
    args
}

// ─── Progress ───────────────────────────────────────────────────────────────

/// Markers we ask yt-dlp to print, so its output is easy to read reliably.
pub const PROGRESS_MARK: &str = "@@P ";
pub const STAGE_MARK: &str = "@@S ";
pub const FILE_MARK: &str = "@@F ";

pub fn progress_args() -> Vec<String> {
    [
        "--newline",
        "--progress",
        "--progress-delta",
        "0.5",
        "--progress-template",
        "download:@@P %(progress.{status,downloaded_bytes,total_bytes,total_bytes_estimate,speed,eta})j %(info.vcodec)s",
        "--progress-template",
        "postprocess:@@S %(progress.postprocessor)s",
        "-O",
        "after_move:@@F %(filepath)s",
    ]
    .map(String::from)
    .to_vec()
}

#[derive(Debug, Clone, PartialEq)]
pub enum Line {
    Progress {
        stage: &'static str,
        percent: f64,
        speed: Option<f64>,
        eta: Option<u64>,
    },
    Finishing,
    File(String),
    Other(String),
}

pub fn parse_line(line: &str) -> Line {
    let line = line.trim_end();
    if let Some(rest) = line.strip_prefix(PROGRESS_MARK) {
        if let Some((json, vcodec)) = rest.rsplit_once(' ') {
            if let Ok(p) = serde_json::from_str::<Value>(json) {
                let downloaded = p.get("downloaded_bytes").and_then(Value::as_f64).unwrap_or(0.0);
                let total = p
                    .get("total_bytes")
                    .and_then(Value::as_f64)
                    .or_else(|| p.get("total_bytes_estimate").and_then(Value::as_f64))
                    .filter(|t| *t > 0.0);
                return Line::Progress {
                    stage: if vcodec == "none" { "audio" } else { "video" },
                    percent: total.map_or(0.0, |t| (100.0 * downloaded / t).clamp(0.0, 100.0)),
                    speed: p.get("speed").and_then(Value::as_f64),
                    eta: p.get("eta").and_then(Value::as_u64),
                };
            }
        }
        Line::Other(line.to_string())
    } else if line.starts_with(STAGE_MARK) {
        Line::Finishing
    } else if let Some(path) = line.strip_prefix(FILE_MARK) {
        Line::File(path.to_string())
    } else {
        Line::Other(line.to_string())
    }
}

// ─── Errors in plain words ──────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct FriendlyError {
    pub message: String,
    pub detail: String,
}

/// The last `ERROR:` line, explained in plain words when it's a known case.
pub fn friendly_error(output: &str) -> FriendlyError {
    let lines: Vec<&str> = output.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
    let detail = lines
        .iter()
        .rev()
        .find(|l| l.starts_with("ERROR:"))
        .or(lines.last())
        .map(|l| l.trim_start_matches("ERROR:").trim().to_string())
        .unwrap_or_default();
    let lower = detail.to_lowercase();
    let known = [
        ("private video", "This video is private."),
        ("confirm your age", "This video is age-restricted, so it can't be downloaded without signing in."),
        ("members-only", "This video is for channel members only."),
        ("join this channel", "This video is for channel members only."),
        ("not a bot", "YouTube asked to confirm you're not a bot. Try again later, or update yt-dlp."),
        ("unavailable", "This video is unavailable."),
        ("has been removed", "This video has been removed."),
        ("http error 403", "YouTube refused the download for now (error 403). Wait a moment and try again. If it keeps happening, update yt-dlp."),
        ("requested format is not available", "That quality isn't available for this video. Try another one."),
        ("getaddrinfo", "Couldn't reach YouTube. Check your internet connection."),
        ("failed to resolve", "Couldn't reach YouTube. Check your internet connection."),
        ("timed out", "YouTube took too long to answer. Check your connection and try again."),
        ("unable to download webpage", "Couldn't reach YouTube. Check your internet connection."),
    ];
    let message = known
        .iter()
        .find(|(needle, _)| lower.contains(needle))
        .map(|(_, message)| message.to_string())
        .unwrap_or_else(|| "yt-dlp couldn't do that. Updating yt-dlp often fixes this.".into());
    FriendlyError { message, detail }
}

// ─── File names ─────────────────────────────────────────────────────────────

/// Make a title safe to use as a Windows/macOS/Linux file name (no extension).
pub fn sanitize_file_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for c in name.chars() {
        match c {
            ':' => out.push_str(" - "),
            '/' | '\\' | '|' => out.push('-'),
            '<' | '>' | '"' | '?' | '*' => {}
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    let collapsed = out.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut cleaned: String = collapsed.chars().take(150).collect();
    while cleaned.ends_with(['.', ' ']) {
        cleaned.pop(); // Windows drops trailing dots and spaces
    }
    if cleaned.is_empty() {
        return "video".into();
    }
    let stem = cleaned.split('.').next().unwrap_or("").to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        cleaned.insert(0, '_');
    }
    cleaned
}

/// yt-dlp reads `%` in an output template as a field; double it to keep it literal.
pub fn escape_output_template(name: &str) -> String {
    name.replace('%', "%%")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_lines_are_read() {
        let line = r#"@@P {"status": "downloading", "downloaded_bytes": 50, "total_bytes": 200, "speed": 1000.5, "eta": 3} av01.0.00M.08"#;
        assert_eq!(
            parse_line(line),
            Line::Progress { stage: "video", percent: 25.0, speed: Some(1000.5), eta: Some(3) }
        );
        let audio = r#"@@P {"status": "downloading", "downloaded_bytes": 10, "total_bytes_estimate": 40} none"#;
        assert!(matches!(parse_line(audio), Line::Progress { stage: "audio", percent, .. } if percent == 25.0));
        let unknown_total = r#"@@P {"status": "downloading", "downloaded_bytes": 10} avc1"#;
        assert!(matches!(parse_line(unknown_total), Line::Progress { percent, .. } if percent == 0.0));
        assert_eq!(parse_line("@@S Merger"), Line::Finishing);
        assert_eq!(parse_line("@@F C:\\Videos\\a b.mp4\r"), Line::File("C:\\Videos\\a b.mp4".into()));
        assert!(matches!(parse_line("WARNING: something"), Line::Other(_)));
    }

    #[test]
    fn errors_get_plain_words() {
        let e = friendly_error("[youtube] x: Downloading\nERROR: [youtube] abc: Private video. Sign in if you've been granted access");
        assert_eq!(e.message, "This video is private.");
        assert!(e.detail.starts_with("[youtube] abc: Private video"));
        let e = friendly_error("ERROR: [youtube] abc: Sign in to confirm your age.");
        assert!(e.message.contains("age-restricted"));
        let e = friendly_error("ERROR: [youtube] aaaaaaaaaaa: This video is unavailable");
        assert_eq!(e.message, "This video is unavailable.");
        let e = friendly_error("ERROR: unable to download video data: HTTP Error 403: Forbidden");
        assert!(e.message.contains("error 403"));
        let e = friendly_error("ERROR: something new");
        assert!(e.message.contains("Updating yt-dlp"));
        assert_eq!(e.detail, "something new");
    }

    #[test]
    fn file_names_are_made_safe() {
        assert_eq!(sanitize_file_name("Me at the zoo"), "Me at the zoo");
        assert_eq!(sanitize_file_name("Part 1: The Start"), "Part 1 - The Start");
        assert_eq!(sanitize_file_name("a/b\\c|d"), "a-b-c-d");
        assert_eq!(sanitize_file_name("What? <Really> \"yes\" *"), "What Really yes");
        assert_eq!(sanitize_file_name("ends with dots..."), "ends with dots");
        assert_eq!(sanitize_file_name("CON"), "_CON");
        assert_eq!(sanitize_file_name("com1.txt"), "_com1.txt");
        assert_eq!(sanitize_file_name("COMPANY"), "COMPANY");
        assert_eq!(sanitize_file_name("???"), "video");
        assert_eq!(sanitize_file_name(&"x".repeat(300)).chars().count(), 150);
        assert_eq!(sanitize_file_name("emoji 🎉 title"), "emoji 🎉 title");
        assert_eq!(escape_output_template("100% real"), "100%% real");
    }

    #[test]
    fn info_is_read_from_ytdlp_json() {
        let json = serde_json::json!({
            "id": "jNQXAC9IVRw", "title": "Me at the zoo", "uploader": "jawed",
            "channel_url": "https://www.youtube.com/channel/UC4QobU6STFB0P71PMvOGN5A",
            "thumbnail": "https://i.ytimg.com/vi/jNQXAC9IVRw/hqdefault.jpg",
            "duration": 19, "upload_date": "20050423", "view_count": 380000000u64,
            "description": "The first video",
            "formats": [
                {"vcodec": "none", "height": null},
                {"vcodec": "avc1", "height": 240, "width": 320},
                {"vcodec": "vp9", "height": 240, "width": 320},
                {"vcodec": "avc1", "height": 1920, "width": 1080}
            ]
        });
        let info = parse_info("https://www.youtube.com/watch?v=jNQXAC9IVRw", &json);
        assert_eq!(info.channel, "jawed");
        assert_eq!(info.upload_date.as_deref(), Some("2005-04-23"));
        assert_eq!(info.heights, vec![1080, 240]); // vertical 1080x1920 counts as 1080
        assert_eq!(info.suggested_file_name, "Me at the zoo");
        assert_eq!(info.duration_seconds, Some(19.0));
        // No sized thumbnails listed: falls back to yt-dlp's own pick.
        assert_eq!(info.thumbnail.as_deref(), Some("https://i.ytimg.com/vi/jNQXAC9IVRw/hqdefault.jpg"));
    }

    #[test]
    fn thumbnail_is_the_smallest_sharp_one() {
        let json = serde_json::json!({
            "thumbnail": "https://i.ytimg.com/vi_webp/x/maxresdefault.webp",
            "thumbnails": [
                {"url": "https://i.ytimg.com/vi/x/mqdefault.jpg", "width": 320},
                {"url": "https://i.ytimg.com/vi/x/hqdefault.jpg", "width": 480},
                {"url": "https://i.ytimg.com/vi/x/hq720.jpg", "width": 1280},
                {"url": "https://i.ytimg.com/vi_webp/x/maxresdefault.webp", "width": 1920},
                {"url": "https://i.ytimg.com/vi/x/sd1.jpg"}
            ]
        });
        assert_eq!(pick_thumbnail(&json).as_deref(), Some("https://i.ytimg.com/vi/x/hqdefault.jpg"));
    }

    #[test]
    fn formats_map_to_documented_options() {
        assert_eq!(format_args(Format::Mp4, Some(720), 192)[1], "res:720,codec:h264:m4a");
        assert_eq!(format_args(Format::Mkv, None, 192)[1], "res");
        assert_eq!(format_args(Format::Webm, Some(1080), 192)[3], "webm/mkv");
        assert!(format_args(Format::M4a, Some(720), 192).contains(&"-x".to_string()));
        // M4A is never converted; MP3 is, at the chosen bitrate.
        assert!(!format_args(Format::M4a, None, 192).contains(&"--audio-quality".to_string()));
        let mp3 = format_args(Format::Mp3, Some(720), 320);
        assert!(mp3.windows(2).any(|w| w == ["--audio-format", "mp3"]));
        assert!(mp3.windows(2).any(|w| w == ["--audio-quality", "320K"]));
        assert!(!mp3.iter().any(|a| a.starts_with("res")), "a video height never reaches an audio download");
    }
}
