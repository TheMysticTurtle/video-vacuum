# Video Vacuum — Project Quick Reference

> Auto-loaded at the start of every session. Keep it short and current.
> It is the **map**, not the territory. The full detail lives in the files below.

## What this project is

A small Windows desktop app (Tauri 2 + React) that downloads YouTube videos
with yt-dlp: paste a link, see the video's details, pick format, quality, file
name and folder, download. It keeps a searchable History drawer. Open source
(GPL-3.0-or-later), free, for people who'd rather not use a terminal. Repo:
`TheMysticTurtle/video-vacuum` (public since 2026-11-22; renamed from `video-saver` the same day).

## The golden rules for this project

1. **Scope discipline:** a plain, simple downloader. Owner: "super simple easy
   stupid". Clips, preview and GIF export are backlog (D-19); don't build them unasked.
2. **"Just download what YouTube sends"** (D-05): no re-encoding.
3. **The page never runs programs or touches files**; validated Rust commands do (D-13).
4. **Licensing:** Video Vacuum is GPL-3.0-or-later (D-20). Every dependency must
   be GPLv3-compatible: MIT / Apache-2.0 / BSD / MPL-2.0 / GPLv3 are fine;
   GPLv2-*only* and proprietary code are not. Check the license and official
   source before adding one, and record it in `docs/TECH_REFERENCE.md`.
5. Follow the owner's global working style (`~/.claude/CLAUDE.md`): confirm
   plans before acting, do only what's asked, best practice by default.
6. **The project's own brand, never YouTube's** (D-09, D-24). Brand images come from
   `media/` via `tools/make-brand-images.ps1`; the look follows system light/dark.

## Current status

- **Read `ROADMAP.md` first.** `PROGRESS.md` is the dated log; `docs/DECISIONS.md` has the why.
- **Where we are (2026-11-22):** Video Vacuum v2.0.0 (desktop app, GPL-3.0) is
  built, tested, merged and pushed; releases come from the manual **Release**
  GitHub Actions workflow (`docs/TECH_REFERENCE.md` §6). Installer smoke-tested.
  The dark-studio look (GPT, then Claude) with the owner's brand art is merged (D-24, D-25).
  **v2.0.0 is released** on GitHub (2026-11-22) and the owner has walked through it.
  The repo is **public**, with `PRIVACY.md` and every commit under the GitHub
  noreply email (D-26). v1 is kept as tag `v1.0.0-local-web`.
  **v2.1.0 is released** (2026-11-23, tag `v2.1.0` at `b8de617`, marked Latest):
  history privacy controls (D-28), MP3 with a chosen bitrate (D-27), the
  author name TheMysticTurtle (D-29), the Settings panel / theme / layout
  polish (D-30), the bottom bar and the diagnostic Log (D-31), and README
  screenshots. The owner tested the installer ("it's beautiful! seems to work
  great").
  **v2.2.0 is released** (2026-11-23, tag `v2.2.0` at `6adc762`, marked Latest):
  - yt-dlp, Deno and a small FFmpeg ship inside the installer (D-32):
    pinned in `src-tauri/tools.json`, fetched by `tools/fetch-tools.ps1`
    into `src-tauri/tools/` (git-ignored; **run it before any Rust build**).
  - The FFmpeg zip lives in the `tools-ffmpeg-9.1` release.
  - The fix for an MP3 deleting a same-name WebM (D-33).
  - The owner tested the installer. Nothing is queued.
- **Author:** always `TheMysticTurtle <101545919+TheMysticTurtle@users.noreply.github.com>`
  (repo config and global), in commits, copyright lines and credits. Never
  "TheMysticTestudine" (another account) and never a personal email.

## Documentation map

| File | Purpose |
|---|---|
| `ROADMAP.md` | What's next. Read first. |
| `PROGRESS.md` | Dated session log. Update last, every session. |
| `docs/DECISIONS.md` | Every locked choice and why (D-xx). |
| `docs/TECH_REFERENCE.md` | Stack, versions, licenses, official sources, gotchas, how to build and test. |
| `README.md` | For users: install and use. |

## Code map

| Path | What |
|---|---|
| `src-tauri/src/lib.rs` | The commands the window calls, app state, startup |
| `src-tauri/src/link.rs` | Pasted text → clean YouTube watch link (tests: every link form) |
| `src-tauri/src/tools.rs` | The bundled tools (D-32): version check, Restore, yt-dlp `-U` |
| `src-tauri/tools.json` | Pinned tool versions, official downloads, SHA-256 |
| `tools/fetch-tools.ps1` | Downloads the pinned tools into `src-tauri/tools/` |
| `.github/workflows/build-ffmpeg.yml` | Builds the trimmed LGPL FFmpeg zip |
| `src-tauri/src/ytdlp.rs` | Video info, format options, progress lines, plain-word errors, safe file names |
| `src-tauri/src/download.rs` | One download: unique name, run, progress, cancel, cleanup |
| `src-tauri/src/store.rs` | History + settings JSON (atomic, versioned) |
| `src-tauri/src/activity.rs` | The diagnostic log: session memory, optional daily files, 30-day cleanup (D-31) |
| `src/App.tsx`, `src/components/` | The window: Link → Video → Save as → status, History drawer |
| `src/api.ts` | Typed calls to the Rust commands |

## Hard constraints to remember

- **Platform:** Windows 10/11 (WebView2). The installer is NSIS, per-user, no admin prompt.
- **Quality gate** before every commit: `cargo clippy --all-targets -- -D warnings`
  + `cargo test` (in `src-tauri/`) + `npm run check`. Real downloads through the
  real window are the main proof (see `docs/TECH_REFERENCE.md`, "Testing").
- **Git:** one `feature/<name>` branch per build. **Never merge into `main`
  until the owner has tried the build themselves and said it's good**
  (owner, 2026-11-23: "do not auto merge branches into my stable main until I
  test it myself and tell you it's good"). Then merge with `--no-ff`. Commits: WHY / WHAT / TEST RESULTS. **Claude commits; the owner pushes.**
- **Test windows stay off the owner's screen** (hidden window + CDP; see the tech reference).
