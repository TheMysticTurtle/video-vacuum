# Project notes — Video Saver

The working notebook for this project: what's done, what's next, how it's built
and why. Read this first when picking the project back up.

## Goal

A super simple, lightweight, self-contained way for anyone to save YouTube
videos with yt-dlp, without touching a terminal. Paste a link, see the video
(thumbnail, creator, clean link), choose format + quality, download. Anyone
should be able to figure it out in 10 seconds.

Standing requirements from the owner: robust, minimal, efficient, no fluff,
by-the-book (official docs), clearly commented, nicely labeled.

## Status — 2026-10-25

**Working and tested.** v1 is complete. Pushed to a **private** GitHub repo,
https://github.com/TheMysticTurtle/video-saver. The owner is testing it before
making it public.

### Done
- [x] Researched yt-dlp README (format sorting, progress template, `-O`, `-J`,
      update channels, JS runtime) and Python `http.server` docs
- [x] `app.py`: local server, link cleaning, info lookup, downloads with
      progress, show-in-folder, update button
- [x] `web/`: single page (HTML + CSS + JS), light/dark, calm neutral look
- [x] `Start.bat` (Windows, offers winget installs) and `start.sh` (macOS/Linux)
- [x] Security hardening for a localhost server (see below)
- [x] Restyle: dropped YouTube-red branding, renamed to **Video Saver**, added
      a one-line trust note under the title
- [x] README, LICENSE (MIT), .gitignore, .gitattributes, git init

### Next / ideas (not started — owner decides)
- [x] Create the GitHub repo and push (private, 2026-10-25)
- [ ] Owner tests it, then decides whether to make the repo public
- [ ] Optional: make the local page installable as a PWA (own icon/window)
- [ ] Optional: playlist support (deliberately left out to keep it simple)

## Architecture

```
Browser page (web/)  ──fetch JSON──▶  app.py (127.0.0.1:8765)  ──subprocess──▶  yt-dlp (+ ffmpeg, deno)
        ▲                                     │
        └──── polls /api/job twice a second ◀─┘ parses yt-dlp's progress lines
```

| File | Role |
|---|---|
| `app.py` | Python standard-library server. Serves the 3 page files, cleans links, runs yt-dlp, tracks download jobs. |
| `web/index.html` | Page structure: Link → Video → Download sections, footer with version + update. |
| `web/app.js` | Page logic: calls the API, shows the video card, progress, messages. |
| `web/style.css` | Neutral palette, one blue accent, follows system light/dark. |
| `Start.bat` / `start.sh` | Check dependencies, then run `app.py`. |

### API (all JSON)

| Method + path | Does |
|---|---|
| `GET /api/version` | yt-dlp version |
| `POST /api/info {url}` | cleans the link, runs `yt-dlp -J`, returns title/channel/thumbnail/duration/heights |
| `POST /api/download {url, format, height}` | starts a job, returns `{job}` |
| `GET /api/job?id=` | job status, percent, speed, ETA, file, error |
| `POST /api/open {job}` | opens the finished file's folder |
| `POST /api/update` | updates yt-dlp (winget upgrade, else `yt-dlp -U`) |

### Security model (localhost server)
- Binds to `127.0.0.1` only, so nothing on the network can reach it.
- **Host header check** (`127.0.0.1:port` / `localhost:port`) blocks DNS rebinding.
- **POSTs need `Origin` = our own origin and `Content-Type: application/json`**,
  so other websites can't trigger downloads or updates (CSRF).
- Static files come from a fixed whitelist, so there's no path traversal.
- yt-dlp gets a list of arguments (no shell), `--` before the URL, and the URL
  is rebuilt from a validated 11-character video ID, so it can't inject options.
- `/api/open` only opens files this server downloaded (looked up by job ID).
- Headers: strict CSP (self + `*.ytimg.com` images), `nosniff`,
  `no-referrer`, `frame-ancestors 'none'`.

## Key decisions (and why)

| Decision | Why |
|---|---|
| Local web app, **not** a hosted site | A static page can't run yt-dlp, and a public download service is a legal/ToS grey area that YouTube also blocks. Each user runs it on their own machine. |
| Python **stdlib only** | No pip, no npm. Cross-platform, readable, nothing to install beyond yt-dlp. |
| Call the `yt-dlp` program, not its Python module | Stays independent of how yt-dlp is installed and lets winget/`-U` update it. |
| `--ignore-config`, `--no-playlist`, `--encoding utf-8` | Predictable results. A `&list=` link means just that video. Emoji titles survive the pipe. |
| Quality via `-S res:N` | yt-dlp README's recommended way to cap resolution. Uses the shorter side, so vertical Shorts line up. |
| MP4 = `-S res:N,codec:h264:m4a` | Plain `ext` sorting picked VP9-in-MP4 at low resolutions (found in testing with ffprobe). H.264 + AAC is what "plays everywhere" actually means. |
| No re-encoding | "Download what YouTube sends." ffmpeg only remuxes. M4A takes YouTube's AAC as-is. |
| Progress via `--progress-template` with `@@` markers + JSON | Reliable parsing, no scraping of human-readable output. |
| Update = `winget upgrade` when winget-installed | yt-dlp README: package-manager installs must update through that manager. Otherwise `-U` (which prints instructions for pip/brew). |
| Polling instead of SSE/WebSockets | Simplest robust option. Twice a second on localhost costs nothing. |
| `Start.bat` doesn't pass `--accept-*-agreements` to winget | The user accepts winget's prompts themselves. |
| Name "Video Saver", no red | YouTube name + red looked like a sketchy clone site and borrows their branding. |

## Testing log

**2026-10-25** (Windows 11, Python 3.14, yt-dlp 2026.10.21)
- Link cleaning: 11 good forms → canonical link, 9 bad/lookalike inputs rejected.
- Security: wrong Host → 403; cross-site Origin → 403; missing Origin → 403;
  form-encoded POST → 403; `../app.py` → 404; bad format → 400.
- Downloads checked with ffprobe:
  - MP4 → h264 + aac
  - WebM → vp9 + opus
  - MKV → av1 + opus
  - M4A → aac
- In-browser: paste → card → download → progress → "Saved" → Show in folder.
  Update button ran `winget upgrade` and correctly reported no upgrade.
- `Start.bat` launched by full path (as a double-click does) → server answered.
  - Note: running `cmd /c Start.bat` from the Claude Code sandbox shell fails
    with "not recognized" because cmd doesn't see the folder. That's a test
    harness quirk, not a launcher bug.
- Visual check in light and dark mode.
