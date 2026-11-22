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
  noreply email (D-26). v1 is kept as tag `v1.0.0-local-web`. Nothing is queued;
  the owner picks from the ROADMAP backlog.
- **Commit email:** always `101545919+TheMysticTurtle@users.noreply.github.com`
  (set in this repo's config and globally). Never commit with a personal email.

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
| `src-tauri/src/tools.rs` | Finding yt-dlp / ffmpeg / deno, winget install and update |
| `src-tauri/src/ytdlp.rs` | Video info, format options, progress lines, plain-word errors, safe file names |
| `src-tauri/src/download.rs` | One download: unique name, run, progress, cancel, cleanup |
| `src-tauri/src/store.rs` | History + settings JSON (atomic, versioned) |
| `src/App.tsx`, `src/components/` | The window: Link → Video → Save as → status, History drawer |
| `src/api.ts` | Typed calls to the Rust commands |

## Hard constraints to remember

- **Platform:** Windows 10/11 (WebView2). The installer is NSIS, per-user, no admin prompt.
- **Quality gate** before every commit: `cargo clippy --all-targets -- -D warnings`
  + `cargo test` (in `src-tauri/`) + `npm run check`. Real downloads through the
  real window are the main proof (see `docs/TECH_REFERENCE.md`, "Testing").
- **Git:** one `feature/<name>` branch per build; merge to `main` with `--no-ff`
  only when tested. Commits: WHY / WHAT / TEST RESULTS. **Claude commits; the owner pushes.**
- **Test windows stay off the owner's screen** (hidden window + CDP; see the tech reference).
