#!/usr/bin/env python3
"""
Video Saver — a tiny local web app around yt-dlp.

How it works
------------
1. This script starts a small web server on http://127.0.0.1 (your computer
   only — nothing on your network or the internet can reach it).
2. It opens the page in your browser. You paste a link, pick a format and a
   quality, and click Download.
3. The server runs the real `yt-dlp` program to do the work and reports
   progress back to the page. Files land in your Downloads folder.

Only the Python standard library is used. External programs required:
yt-dlp, ffmpeg and deno (all installed by `winget install yt-dlp.yt-dlp`).

References followed:
- yt-dlp README: format sorting (-S), --progress-template, -O/--print, -J,
  update channels        https://github.com/yt-dlp/yt-dlp#readme
- Python http.server     https://docs.python.org/3/library/http.server.html
- Python subprocess      https://docs.python.org/3/library/subprocess.html
"""

from __future__ import annotations

import json
import os
import re
import secrets
import shutil
import subprocess
import sys
import threading
import webbrowser
from dataclasses import dataclass, asdict
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import parse_qs, urlsplit

# ─── Settings ────────────────────────────────────────────────────────────────

HOST = "127.0.0.1"                     # localhost only — never "0.0.0.0"
FIRST_PORT = 8765                      # tries 8765, 8766, … if one is busy
PORT_ATTEMPTS = 10
WEB_DIR = Path(__file__).resolve().parent / "web"
DOWNLOAD_DIR = Path.home() / "Downloads"

# The only files the server will ever hand out (no directory browsing).
STATIC_FILES = {
    "/":           ("index.html", "text/html; charset=utf-8"),
    "/index.html": ("index.html", "text/html; charset=utf-8"),
    "/app.js":     ("app.js",     "text/javascript; charset=utf-8"),
    "/style.css":  ("style.css",  "text/css; charset=utf-8"),
}

# Format menu shown in the page → yt-dlp options (filled in by format_args).
FORMATS = ("mp4", "webm", "mkv", "m4a")

# Hide the console window that Windows would otherwise flash for each yt-dlp run.
NO_WINDOW = subprocess.CREATE_NO_WINDOW if sys.platform == "win32" else 0

# Options used on every yt-dlp call:
#   --ignore-config   predictable results, regardless of any yt-dlp config file
#   --no-playlist     a link like watch?v=X&list=Y means "just video X"
#   --encoding utf-8  titles with emoji / non-Latin text come through intact
YTDLP_BASE = ["--ignore-config", "--no-playlist", "--encoding", "utf-8"]


# ─── Finding yt-dlp ──────────────────────────────────────────────────────────

YTDLP = shutil.which("yt-dlp")


def run_ytdlp(args: list[str], timeout: int) -> subprocess.CompletedProcess:
    """Run yt-dlp once and capture its output as text."""
    return subprocess.run(
        [YTDLP, *args], capture_output=True, text=True, encoding="utf-8",
        errors="replace", timeout=timeout, creationflags=NO_WINDOW,
    )


def ytdlp_version() -> str:
    try:
        return run_ytdlp(["--version"], timeout=30).stdout.strip() or "unknown"
    except (OSError, subprocess.SubprocessError):
        return "unknown"


def installed_with_winget() -> bool:
    """winget-installed yt-dlp lives under …\\Microsoft\\WinGet\\…"""
    return "\\winget\\" in str(Path(YTDLP).resolve()).lower()


def update_command() -> list[str]:
    """
    The right way to update depends on how yt-dlp was installed (yt-dlp README,
    "UPDATE"): release binaries update themselves with -U; package-manager
    installs must be updated by that package manager.
    """
    if installed_with_winget():
        return ["winget", "upgrade", "--id", "yt-dlp.yt-dlp", "--exact",
                "--source", "winget", "--disable-interactivity"]
    # Release binary: updates itself. pip/brew installs: yt-dlp prints which
    # command to run instead, and we show that message to the user.
    return [YTDLP, "-U"]


# ─── Cleaning up pasted links ────────────────────────────────────────────────

VIDEO_ID = re.compile(r"[A-Za-z0-9_-]{11}")            # every YouTube video ID
LINK_IN_TEXT = re.compile(r"\S*(?:youtube(?:-nocookie)?\.com|youtu\.be)/\S*", re.I)
YOUTUBE_HOSTS = {
    "youtube.com", "www.youtube.com", "m.youtube.com", "music.youtube.com",
    "youtube-nocookie.com", "www.youtube-nocookie.com",
}
PATH_STYLE_PREFIXES = {"shorts", "live", "embed", "v", "e"}   # /shorts/<id> etc.


def clean_youtube_url(text: str) -> str | None:
    """
    Turn whatever was pasted into the plain link
        https://www.youtube.com/watch?v=<id>
    dropping tracking/extra bits like ?si=, &t=, &list=, &pp=.
    Returns None if it isn't a YouTube video link.
    """
    text = (text or "").strip()
    if VIDEO_ID.fullmatch(text):                     # a bare video ID
        return f"https://www.youtube.com/watch?v={text}"

    found = LINK_IN_TEXT.search(text)                # the link inside the paste
    if not found:
        return None
    link = found.group(0)
    if not re.match(r"https?://", link, re.I):
        link = "https://" + link

    parts = urlsplit(link)
    host = (parts.hostname or "").lower()
    segments = [s for s in parts.path.split("/") if s]
    video_id = ""

    if host == "youtu.be" and segments:                          # youtu.be/<id>
        video_id = segments[0]
    elif host in YOUTUBE_HOSTS:
        if parts.path == "/watch":                               # /watch?v=<id>
            video_id = parse_qs(parts.query).get("v", [""])[0]
        elif len(segments) >= 2 and segments[0] in PATH_STYLE_PREFIXES:
            video_id = segments[1]                               # /shorts/<id>

    if VIDEO_ID.fullmatch(video_id):
        return f"https://www.youtube.com/watch?v={video_id}"
    return None


# ─── Getting video info ──────────────────────────────────────────────────────

def video_info(url: str) -> dict:
    """Ask yt-dlp about the video (-J = print info as JSON, download nothing)."""
    result = run_ytdlp([*YTDLP_BASE, "-J", "--", url], timeout=120)
    if result.returncode != 0:
        raise RuntimeError(last_error(result.stderr) or "yt-dlp could not read that video.")
    info = json.loads(result.stdout)

    # Every video height on offer, e.g. [2160, 1440, 1080, 720, 480, 360, 240, 144].
    # (Uses the shorter side so vertical Shorts line up with normal videos,
    # matching how yt-dlp's own "res" sort measures resolution.)
    heights = sorted({
        min(f["width"], f["height"]) if f.get("width") else f["height"]
        for f in info.get("formats", [])
        if f.get("vcodec") not in (None, "none") and f.get("height")
    }, reverse=True)

    return {
        "url":         url,
        "title":       info.get("title") or "Untitled",
        "channel":     info.get("channel") or info.get("uploader") or "Unknown",
        "channel_url": info.get("channel_url") or info.get("uploader_url") or "",
        "thumbnail":   info.get("thumbnail") or "",
        "duration":    format_duration(info.get("duration")),
        "heights":     heights,
    }


def format_duration(seconds: float | None) -> str:
    """19 → "0:19", 3725 → "1:02:05"."""
    if not seconds:
        return ""
    minutes, secs = divmod(int(seconds), 60)
    hours, minutes = divmod(minutes, 60)
    return f"{hours}:{minutes:02}:{secs:02}" if hours else f"{minutes}:{secs:02}"


def last_error(output: str) -> str:
    """Pick the most useful line ("ERROR: …") out of yt-dlp's output."""
    lines = [l.strip() for l in (output or "").splitlines() if l.strip()]
    errors = [l for l in lines if l.startswith("ERROR:")]
    return (errors or lines or [""])[-1].removeprefix("ERROR:").strip()


# ─── Downloading ─────────────────────────────────────────────────────────────

def format_args(fmt: str, height: int | None) -> list[str]:
    """
    yt-dlp options for each menu choice. YouTube sends video and audio as
    separate streams; yt-dlp grabs the best pair and ffmpeg joins them into
    the chosen container *without re-encoding* (no quality loss).

    "-S res:720" = best quality that is no larger than 720p (yt-dlp README,
    "Sorting Formats"). The fields after it pick between streams of that size.
    """
    res = f"res:{height}" if height else "res"
    if fmt == "mp4":    # most compatible: H.264 + AAC (YouTube only offers H.264
                        # up to 1080p; above that it's VP9/AV1, still in an .mp4)
        return ["-S", f"{res},codec:h264:m4a", "--merge-output-format", "mp4", "--remux-video", "mp4"]
    if fmt == "webm":   # VP9/AV1 + Opus; falls back to .mkv if a WebM pair isn't offered
        return ["-S", f"{res},ext:webm:webm", "--merge-output-format", "webm/mkv"]
    if fmt == "mkv":    # holds anything — always the very best streams as-is
        return ["-S", res, "--merge-output-format", "mkv", "--remux-video", "mkv"]
    if fmt == "m4a":    # audio only; YouTube's AAC audio is already .m4a, so no conversion
        return ["-f", "ba[ext=m4a]/ba/b", "-x", "--audio-format", "m4a"]
    raise ValueError(f"Unknown format: {fmt}")


# Markers we ask yt-dlp to print so its progress is easy to read reliably.
PROGRESS_MARK, STAGE_MARK, FILE_MARK = "@@P ", "@@S ", "@@F "
PROGRESS_ARGS = [
    "--newline", "--progress", "--progress-delta", "0.5",
    "--progress-template",
    "download:" + PROGRESS_MARK +
    "%(progress.{status,downloaded_bytes,total_bytes,total_bytes_estimate,speed,eta})j"
    " %(info.vcodec)s",
    "--progress-template", "postprocess:" + STAGE_MARK + "%(progress.postprocessor)s",
    "-O", "after_move:" + FILE_MARK + "%(filepath)s",       # final file path
]


@dataclass
class Job:
    """One download, as reported to the page."""
    status: str = "starting"   # starting → downloading → finishing → done | error
    stage: str = ""            # "video" / "audio" while downloading
    percent: float = 0.0
    speed: float | None = None # bytes per second
    eta: int | None = None     # seconds left
    file: str = ""
    error: str = ""


jobs: dict[str, Job] = {}
jobs_lock = threading.Lock()


def start_download(url: str, fmt: str, height: int | None) -> str:
    job_id = secrets.token_urlsafe(8)
    with jobs_lock:
        jobs[job_id] = Job()
    threading.Thread(target=run_download, args=(job_id, url, fmt, height), daemon=True).start()
    return job_id


def run_download(job_id: str, url: str, fmt: str, height: int | None) -> None:
    job = jobs[job_id]
    DOWNLOAD_DIR.mkdir(parents=True, exist_ok=True)
    cmd = [YTDLP, *YTDLP_BASE, *PROGRESS_ARGS, *format_args(fmt, height),
           "-P", str(DOWNLOAD_DIR), "--", url]
    error_lines: list[str] = []
    try:
        proc = subprocess.Popen(
            cmd, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
            encoding="utf-8", errors="replace", creationflags=NO_WINDOW,
        )
        for line in proc.stdout:
            line = line.rstrip()
            with jobs_lock:
                if line.startswith(PROGRESS_MARK):
                    data_json, _, vcodec = line[len(PROGRESS_MARK):].rpartition(" ")
                    p = json.loads(data_json)
                    total = p.get("total_bytes") or p.get("total_bytes_estimate")
                    job.status = "downloading"
                    job.stage = "audio" if vcodec == "none" else "video"
                    job.percent = round(100 * p["downloaded_bytes"] / total, 1) if total else 0.0
                    job.speed, job.eta = p.get("speed"), p.get("eta")
                elif line.startswith(STAGE_MARK):
                    job.status = "finishing"       # ffmpeg joining video + audio
                elif line.startswith(FILE_MARK):
                    job.file = line[len(FILE_MARK):]
                else:
                    error_lines.append(line)       # warnings / errors
        proc.wait()
        with jobs_lock:
            if proc.returncode == 0 and job.file:
                job.status, job.percent = "done", 100.0
            else:
                job.status = "error"
                job.error = last_error("\n".join(error_lines)) or "Download failed."
    except Exception as exc:                       # never leave the page hanging
        with jobs_lock:
            job.status, job.error = "error", str(exc)


def show_in_folder(path: str) -> None:
    """Open the file's folder (with the file highlighted on Windows/macOS)."""
    if sys.platform == "win32":
        subprocess.Popen(["explorer", "/select,", path])
    elif sys.platform == "darwin":
        subprocess.Popen(["open", "-R", path])
    else:
        subprocess.Popen(["xdg-open", str(Path(path).parent)])


# ─── Web server ──────────────────────────────────────────────────────────────

# Browser security headers for every response.
SECURITY_HEADERS = {
    # The page may only load its own files, plus thumbnails from YouTube's image CDN.
    "Content-Security-Policy":
        "default-src 'self'; img-src 'self' https://*.ytimg.com; "
        "frame-ancestors 'none'; base-uri 'none'; form-action 'none'",
    "X-Content-Type-Options": "nosniff",
    "Referrer-Policy": "no-referrer",
    "Cache-Control": "no-store",
}


class Handler(BaseHTTPRequestHandler):
    server_version = "VideoSaver"

    # ── Safety checks ──
    def allowed_host(self) -> bool:
        """
        Only answer requests addressed to this machine by name. This blocks
        "DNS rebinding", where a website tricks the browser into talking to
        a local server under a different hostname.
        """
        port = self.server.server_address[1]
        return self.headers.get("Host") in (f"127.0.0.1:{port}", f"localhost:{port}")

    def same_origin_json(self) -> bool:
        """
        Actions (POST) must come from our own page. Browsers always send the
        Origin header on POST, so another website can't trigger downloads or
        updates. Requiring JSON adds a second barrier (cross-site JSON POSTs
        need CORS permission, which this server never grants).
        """
        origin_ok = self.headers.get("Origin") == f"http://{self.headers.get('Host')}"
        json_ok = (self.headers.get("Content-Type") or "").startswith("application/json")
        return origin_ok and json_ok

    # ── Responses ──
    def send(self, status: int, body: bytes, content_type: str) -> None:
        self.send_response(status)
        self.send_header("Content-Type", content_type)
        self.send_header("Content-Length", str(len(body)))
        for name, value in SECURITY_HEADERS.items():
            self.send_header(name, value)
        self.end_headers()
        self.wfile.write(body)

    def send_json(self, data: dict, status: int = 200) -> None:
        self.send(status, json.dumps(data).encode(), "application/json")

    def read_json(self) -> dict:
        length = int(self.headers.get("Content-Length") or 0)
        if not 0 < length <= 10_000:
            return {}
        try:
            data = json.loads(self.rfile.read(length))
            return data if isinstance(data, dict) else {}
        except ValueError:
            return {}

    def log_message(self, format, *args) -> None:
        pass  # keep the console quiet; errors are shown in the page

    # ── GET: page files, job progress, version ──
    def do_GET(self) -> None:
        if not self.allowed_host():
            return self.send_json({"error": "Forbidden"}, 403)
        parts = urlsplit(self.path)

        if parts.path in STATIC_FILES:
            name, content_type = STATIC_FILES[parts.path]
            return self.send(200, (WEB_DIR / name).read_bytes(), content_type)

        if parts.path == "/api/job":
            job_id = parse_qs(parts.query).get("id", [""])[0]
            with jobs_lock:
                job = jobs.get(job_id)
                data = asdict(job) if job else None
            if data is None:
                return self.send_json({"error": "Unknown download."}, 404)
            return self.send_json(data)

        if parts.path == "/api/version":
            return self.send_json({"version": ytdlp_version()})

        self.send_json({"error": "Not found"}, 404)

    # ── POST: actions ──
    def do_POST(self) -> None:
        if not (self.allowed_host() and self.same_origin_json()):
            return self.send_json({"error": "Forbidden"}, 403)
        body = self.read_json()
        path = urlsplit(self.path).path

        try:
            if path == "/api/info":
                url = clean_youtube_url(str(body.get("url", "")))
                if not url:
                    return self.send_json({"error": "That doesn't look like a YouTube video link."}, 400)
                return self.send_json(video_info(url))

            if path == "/api/download":
                url = clean_youtube_url(str(body.get("url", "")))
                fmt = body.get("format")
                height = body.get("height")       # a number, or null for "Best"
                if not url or fmt not in FORMATS:
                    return self.send_json({"error": "Invalid request."}, 400)
                if height is not None and not (isinstance(height, int) and 1 <= height <= 10_000):
                    return self.send_json({"error": "Invalid quality."}, 400)
                return self.send_json({"job": start_download(url, fmt, height)})

            if path == "/api/open":
                with jobs_lock:
                    job = jobs.get(str(body.get("job", "")))
                    file = job.file if job else ""
                if not file:                      # only files we downloaded ourselves
                    return self.send_json({"error": "Unknown download."}, 404)
                show_in_folder(file)
                return self.send_json({"ok": True})

            if path == "/api/update":
                cmd = update_command()
                result = subprocess.run(
                    cmd, capture_output=True, text=True, encoding="utf-8",
                    errors="replace", timeout=600, creationflags=NO_WINDOW,
                )
                output = (result.stdout + result.stderr).strip()
                return self.send_json({"version": ytdlp_version(), "output": output[-2000:]})

        except subprocess.TimeoutExpired:
            return self.send_json({"error": "yt-dlp took too long to respond."}, 504)
        except Exception as exc:
            return self.send_json({"error": str(exc)}, 500)

        self.send_json({"error": "Not found"}, 404)


# ─── Start ───────────────────────────────────────────────────────────────────

def make_server() -> ThreadingHTTPServer:
    """Use the first free port from FIRST_PORT upward."""
    for port in range(FIRST_PORT, FIRST_PORT + PORT_ATTEMPTS):
        try:
            return ThreadingHTTPServer((HOST, port), Handler)
        except OSError:
            continue
    sys.exit(f"No free port between {FIRST_PORT} and {FIRST_PORT + PORT_ATTEMPTS - 1}.")


def main() -> None:
    if not YTDLP:
        sys.exit("yt-dlp was not found. Run Start.bat (Windows) or see README.md.")
    server = make_server()
    url = f"http://{HOST}:{server.server_address[1]}/"
    print(f"Video Saver is running at {url}")
    print(f"Files are saved to {DOWNLOAD_DIR}")
    print("Close this window (or press Ctrl+C) to stop it.")
    webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        server.server_close()


if __name__ == "__main__":
    main()
