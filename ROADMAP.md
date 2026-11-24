# Video Vacuum — Roadmap

> **The one map of what's next.** The owner decides what moves.
> - **Why** something is the way it is → `docs/DECISIONS.md` (D-numbers).
> - **What happened when** → `PROGRESS.md`.
>
> Last full update: **2026-11-22**. Status key: `[ ]` to do · `[~]` in progress · `[x]` done · `[?]` waiting on the owner.

## 1. Now

| | What | State |
|---|---|---|
| `[x]` | **v2 desktop app, basic downloader** (owner 2026-11-21: "go ahead and make the basic downloader app as it is without the gif download"). D-11 to D-21; merged into `main` 2026-11-22 | built and tested |
| `[x]` | **Pushed** `main` + tag `v1.0.0-local-web` (2026-11-22) | done |
| `[x]` | **Release workflow** `.github/workflows/release.yml` (owner 2026-11-22: "builds it. tests it. and makes a release when i run it manually") | done |
| `[x]` | **First release: v2.0.0** published 2026-11-22 by the Release workflow (run 45001774325, 20 min, every step passed); installer `Video.Vacuum_2.0.0_x64-setup.exe` 2.56 MB | done |
| `[x]` | **Owner's hands-on walkthrough** of the released installer (owner 2026-11-22: "i already did the walkthrough") | done |
| `[x]` | **Privacy statement** `PRIVACY.md` + README section (owner: "this thing doesnt collect any info right? do we have a privacy statement?") | done |
| `[x]` | **Repo made public** 2026-11-22 (owner: "nah i want the repo to be public xD"), after rewriting commit emails to the GitHub noreply address (D-26) | done |
| `[x]` | **History privacy** (D-28): Clear history (asks first), Keep history on/off, Remove per entry | built and tested on `feature/history-privacy-mp3` |
| `[x]` | **Settings panel and layout polish** (D-30): Settings button with folder, theme (System / Light / Dark) and history controls; history controls also above the History list; bigger section titles; bordered description with an arrow toggle; 1000 x 960 window that shrinks to fit small screens | built and tested on `feature/history-privacy-mp3` |
| `[x]` | **MP3 with a chosen bitrate** (D-27): 128 / 192 / 256 / 320 kbps, 192 default | built and tested on `feature/history-privacy-mp3` |
| `[x]` | **Author name is TheMysticTurtle** everywhere (D-29) | done |
| `[x]` | **Bottom bar and diagnostic Log** (D-31): engine status, Update yt-dlp, Privacy Policy and GitHub pinned at the bottom; a Log panel docked above it (drag to resize), session-only or saved for 30 days; History panel re-laid out (search, then Keep/Clear on one row) | built and tested on `feature/history-privacy-mp3` |
| `[x]` | **Owner tested the v2.1.0 installer** (2026-11-23: "it's beautiful! seems to work great") and asked to merge and push | done |
| `[x]` | **README screenshots**: the owner's five demo images (start, video loaded, History, Settings, Log) in `docs/images/screenshots/`, with a step-by-step "Use it" | done |
| `[x]` | **Merged into `main` and pushed** (2026-11-23) | done |
| `[x]` | **Release v2.1.0** published 2026-11-23 by the Release workflow (run 45275597949, 4 min with the Rust cache, every step passed); tag `v2.1.0` at `b8de617`; installer `Video.Vacuum_2.1.0_x64-setup.exe` 2.7 MB; marked Latest | done |
| `[?]` | **SmartScreen "incorrect warning" report** for the installer link. Filled in 2026-11-22 except name, email, street address and the CAPTCHA, which only the owner can do; then Submit | waiting on the owner |

## 2. Next

Nothing queued. The owner picks from the backlog.

**Done 2026-11-23: v2.2.0, the tools ship inside the installer** (D-32):

| | What | State |
|---|---|---|
| `[x]` | Spike: trimmed FFmpeg 4.9 + 4.6 MB; 10/10 formats with bundled tools only; `yt-dlp -U` in place | done |
| `[x]` | Build FFmpeg workflow (run 45328682652); zip pinned in `src-tauri/tools.json` | done |
| `[x]` | App: bundled `tools` folder, version check, Restore (checksum-checked), Update yt-dlp via `-U`; winget code removed | built and tested in a hidden window |
| `[x]` | Release workflow fetches the pinned tools; `tools/fetch-tools.ps1` for local builds | done |
| `[x]` | **Fix (D-33):** an MP3 download could delete a same-name WebM (bug in released v2.1.0) | fixed and tested |
| `[x]` | **Owner tested the v2.2.0 installer** ("that seemed to work great! … i downloaded each file type"); merged into `main` (`6adc762`) | done |
| `[x]` | **Published `tools-ffmpeg-9.1`**: the tested zip, not marked Latest; the public download's SHA-256 matches `tools.json` | done |
| `[x]` | **Released v2.2.0** (run 45335023329, 5 min, every step passed; the three tools fetched and checksum-checked on the runner); tag `v2.2.0` at `6adc762`; installer `Video.Vacuum_2.2.0_x64-setup.exe` 58.6 MB; marked Latest | done |
| `[x]` | Deleted the `spike/bundled-tools` branch | done |

## 3. Milestones

- `[x]` v1.0.0 local web edition (2026-10-25, tag `v1.0.0-local-web`)
- `[x]` v2.0.0 desktop app: paste, card, format/quality/name/folder, progress + cancel, History drawer with default folder and search, update button, tools check, installer
- `[x]` Public release: v2.0.0 on GitHub, repo public (2026-11-22)
- `[x]` v2.1.0: history privacy, MP3, Settings and theme, bottom bar and Log, README walkthrough (2026-11-23)
- `[x]` v2.2.0: everything in one installer (yt-dlp, Deno, small FFmpeg), version check and Restore, the MP3/WebM fix (2026-11-23)

## 4. Polish checklist

- `[ ]` Code signing, so Windows' "Windows protected your PC" warning goes away (costs money every year; the owner's call). Without it, each new version starts with no SmartScreen reputation, and Windows 11 Smart App Control blocks it outright for people who have that turned on.
- `[ ]` Optional: submit the installer itself to Microsoft Security Intelligence as a "Software developer" (the file-reputation route; the SmartScreen form above is the website route).

## 5. Decisions waiting on the owner

- **Upload `docs/images/social-preview.jpg`** in GitHub → Settings → Social preview (GitHub has no API for it). Still the default image as of 2026-11-22.
- **GitHub repo description** still says "A tiny local web UI for yt-dlp", which describes v1. Suggested: "Paste a YouTube link, pick format and quality, download. A small Windows app for yt-dlp; runs on your computer, nothing uploaded."
- Optional: a simplified 16 px icon (the detailed logo blurs at title-bar size).

## 6. Backlog (ideas not built now)

- **Video vault / library** (owner 2026-11-22: "what if we made a thing where you could create a video vault similar to obsidian where your videos end up with their tags and you can apply custom ones and group them into folders by category and such to search for them"). Not designed yet. Starting points:
  - yt-dlp's `-J` already returns YouTube's own `tags` and `categories` for each video.
  - The History drawer and `history.json` (D-18) are the natural seed.
  - An open question to decide together: keep the library inside the app, or write one Markdown note per video (with tags) into a folder, so it can be browsed in Obsidian too.

- **Preview and download a section** (owner 2026-11-21: "It'd be really slick if you could preview it in the app and pick a section or chunk to download"). Research done 2026-11-21:
  - yt-dlp `--download-sections "*START-END"` (needs ffmpeg); `--force-keyframes-at-cuts` for frame-exact cuts (re-encodes, slower).
  - YouTube's embedded player may refuse to play inside app web views ("Error 153", a Referer problem seen in MAUI, Capacitor and Android). Not yet tested in Tauri. Fallbacks: play a direct low-res stream, YouTube's storyboard thumbnails, or a small preview download.
  - Where it plugs in: the thumbnail spot in `src/components/VideoCard.tsx`; extra options next to `format_args` in `src-tauri/src/ytdlp.rs` (D-19).
- **Export a section as a GIF** (owner 2026-11-21). FFmpeg two-pass `palettegen` + `paletteuse`. Lessons from Movement Analyzer: cap the length (GIFs get huge with video content), show an estimated size, and write to a temporary file that's renamed only when finished.
- **"Send to Video Vacuum" browser extension** that only passes the current tab's link to the app (no downloading in the extension).
- Playlist support (left out on purpose to keep it simple).
- Offer a YouTube link already on the clipboard when the window opens.
- **Updates from inside the app** with Tauri's official updater plugin (signed with its own free key, not a paid certificate). Today a new version is installed by downloading the installer again; history and settings are kept.
- Clear the window's cached thumbnails along with Clear history (needs a check of Tauri's API first).
