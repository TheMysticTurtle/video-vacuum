//! Turning whatever the user pasted into the plain link
//! `https://www.youtube.com/watch?v=<id>`, dropping extras like `?si=`, `&t=`,
//! `&list=` and `&pp=`. Anything that isn't a YouTube video gives `None`.
//!
//! The link is rebuilt from a validated 11-character video ID, so nothing the
//! user typed is ever passed to yt-dlp as-is.

use url::Url;

/// Hosts that serve `/watch?v=<id>` or `/shorts/<id>` style links.
const YOUTUBE_HOSTS: &[&str] = &[
    "youtube.com",
    "www.youtube.com",
    "m.youtube.com",
    "music.youtube.com",
    "youtube-nocookie.com",
    "www.youtube-nocookie.com",
];

/// Path styles whose second segment is the video ID: `/shorts/<id>`, `/live/<id>`…
const PATH_STYLE_PREFIXES: &[&str] = &["shorts", "live", "embed", "v", "e"];

/// Every YouTube video ID is 11 characters of `A-Z a-z 0-9 - _`.
pub fn is_video_id(text: &str) -> bool {
    text.len() == 11
        && text
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

pub fn watch_url(video_id: &str) -> String {
    format!("https://www.youtube.com/watch?v={video_id}")
}

pub fn clean_youtube_url(pasted: &str) -> Option<String> {
    let text = pasted.trim();
    if is_video_id(text) {
        return Some(watch_url(text)); // a bare video ID
    }

    // The link inside the paste ("check this out https://youtu.be/... lol").
    let token = text.split_whitespace().find(|word| {
        let lower = word.to_ascii_lowercase();
        lower.contains("youtube.com/")
            || lower.contains("youtube-nocookie.com/")
            || lower.contains("youtu.be/")
    })?;
    let lower = token.to_ascii_lowercase();
    let scheme_start = [lower.find("http://"), lower.find("https://")]
        .into_iter()
        .flatten()
        .min();
    let link = match scheme_start {
        Some(start) => token[start..].to_string(),
        None => format!("https://{token}"),
    };

    let parsed = Url::parse(&link).ok()?;
    let host = parsed.host_str()?.to_ascii_lowercase();
    let segments: Vec<&str> = parsed
        .path_segments()
        .map(|parts| parts.filter(|part| !part.is_empty()).collect())
        .unwrap_or_default();

    let video_id = if host == "youtu.be" {
        segments.first()?.to_string() // youtu.be/<id>
    } else if YOUTUBE_HOSTS.contains(&host.as_str()) {
        if parsed.path() == "/watch" {
            parsed
                .query_pairs()
                .find(|(key, _)| key == "v")
                .map(|(_, value)| value.into_owned())? // /watch?v=<id>
        } else if segments.len() >= 2 && PATH_STYLE_PREFIXES.contains(&segments[0]) {
            segments[1].to_string() // /shorts/<id>
        } else {
            return None;
        }
    } else {
        return None; // lookalikes such as youtube.com.evil.com
    };

    is_video_id(&video_id).then(|| watch_url(&video_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLEAN: &str = "https://www.youtube.com/watch?v=jNQXAC9IVRw";

    #[test]
    fn accepts_every_youtube_link_form() {
        for input in [
            "https://www.youtube.com/watch?v=jNQXAC9IVRw&list=PL123&index=2&pp=abc",
            "https://youtu.be/jNQXAC9IVRw?si=AbCdEf123&t=42",
            "youtu.be/jNQXAC9IVRw",
            "https://www.youtube.com/shorts/jNQXAC9IVRw?feature=share",
            "https://m.youtube.com/watch?app=desktop&v=jNQXAC9IVRw",
            "https://music.youtube.com/watch?v=jNQXAC9IVRw&si=x",
            "https://www.youtube.com/live/jNQXAC9IVRw?si=x",
            "https://www.youtube-nocookie.com/embed/jNQXAC9IVRw",
            "  check this out https://youtu.be/jNQXAC9IVRw?si=zz  lol",
            "jNQXAC9IVRw",
            "HTTPS://WWW.YOUTUBE.COM/watch?v=jNQXAC9IVRw",
            "see:https://youtu.be/jNQXAC9IVRw",
        ] {
            assert_eq!(clean_youtube_url(input).as_deref(), Some(CLEAN), "input: {input}");
        }
    }

    #[test]
    fn rejects_non_videos_and_lookalikes() {
        for input in [
            "https://evil.com/youtube.com/watch?v=jNQXAC9IVRw",
            "https://notyoutube.com/watch?v=jNQXAC9IVRw",
            "https://youtube.com.evil.com/watch?v=jNQXAC9IVRw",
            "https://www.youtube.com/@jawed",
            "https://www.youtube.com/playlist?list=PL123",
            "https://youtu.be/short",
            "https://www.youtube.com/watch?v=jNQXAC9IVRw;rm",
            "",
            "hello",
        ] {
            assert_eq!(clean_youtube_url(input), None, "input: {input}");
        }
    }
}
