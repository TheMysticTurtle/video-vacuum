# Video Saver — Progress Log

Newest on top. Forward-looking items live in `ROADMAP.md`; the why lives in `docs/DECISIONS.md`.

### Session 2026-11-21/22 — v2: the desktop app
**Goal:** turn Video Saver into a proper installable app. Owner: "lets stamp what
we have as the old local PWA version. And make it a proper little app."

**What we did / decided:**
- Fixed v1's Windows port-sharing bug (`SO_EXCLUSIVEADDRUSE`), merged into
  `main` (`c431c22`), then tagged the final v1 as `v1.0.0-local-web`.
- Chose a desktop app over a PWA or browser extension (D-11), and Tauri 2 +
  React (D-12) after looking at Palbox Studio, the vault and how other yt-dlp
  GUIs are built.
- Owner narrowed the scope: "make the basic downloader app as it is without the
  gif download … Lets make sure we don't close that door on ourselves." Preview,
  clips and GIF export went to the backlog with their research (D-19).
- Built on `feature/desktop-app`:
  - **Rust backend:** link cleanup ported from v1; tool detection; video info;
    downloads streamed over a Tauri Channel; cancel that ends the whole process
    tree; History and settings JSON.
  - **React window:** link, video card (thumbnail, title, creator, date, length,
    views, description); format, quality, file name and folder; progress with
    Cancel; Open / Show in folder / Download another; a right-side History
    drawer with a ⚙ default-folder menu, search, and greyed-out moved files.
  - Icon, NSIS per-user installer.
  - v1 files removed from the branch; they're preserved in the tag.
- Replaced `NOTES.md` with the vault's software-project document set
  (`CLAUDE.md`, `ROADMAP.md`, `PROGRESS.md`, `docs/DECISIONS.md`, `docs/TECH_REFERENCE.md`).

**Test results:**
- `cargo test`: 14 passed; `cargo clippy --all-targets -D warnings`: clean; `tsc --noEmit`: clean.
- **Real app, real downloads**, driven through the hidden window over CDP (see TECH_REFERENCE "Testing"):
  - A messy paste (`lol check https://youtu.be/jNQXAC9IVRw?si=AbC&t=4 nice`)
    gave the clean link and the correct card, with 240p/144p offered.
  - Each format checked with ffprobe:
    - MP4 144p → h264 + aac
    - WebM 240p → vp9 + opus
    - M4A → aac
    - MP4 best → saved
  - Cancel at 4% of a 4K MKV → "Cancelled. Nothing was kept.", no partial
    files, no yt-dlp/ffmpeg processes left.
  - Unavailable video → "This video is unavailable." (after a fix, below).
  - Open on a moved file → "This file has been moved or deleted."; `open_link`
    to a non-YouTube URL → refused; a non-existent default folder → refused.
  - Update yt-dlp: winget really upgraded 2026.10.21 → 2026.10.26 and the app showed the new version.
  - History drawer: entries with thumbnail, format, quality, size and date;
    search "zoo m4a" found only the M4A; a moved file showed greyed out with "Moved from …".
- License scan: 251 Rust crates + 5 npm packages. All permissive plus 5
  MPL-2.0; no GPL/AGPL; all compatible with GPL-3.0 (D-20).
- Installer `Video Vacuum_2.0.0_x64-setup.exe` (1.7 MB). Silent per-user
  install (`/S /NS`) gave exit 0, files in `%LOCALAPPDATA%\Video Vacuum`, an
  uninstall entry "Video Vacuum 2.0.0 / TheMysticTurtle", and no desktop
  shortcut. The installed release build (strict CSP, bundled assets) loaded,
  called the backend (yt-dlp version shown), and looked up a video with its
  thumbnail. Silent uninstall gave exit 0; the folder and uninstall entry were
  gone. This ran inside the Claude app's redirected AppData; the owner's
  double-click is the real-world test.

**Surprises and fixes:**
- "This video is unavailable" wasn't matched (the needle was "video
  unavailable"), so it now matches "unavailable".
- Audio downloads briefly said "Downloading video…"; the window now shows "Starting…" until real progress arrives.
- Naming counted *any* extension as taken, so an `.m4a` pushed a new MP4 to "(3)".
  Now only the same file type counts, and cleanup uses a before-snapshot (D-16).
- YouTube answered one rapid-fire test download with HTTP 403. A retry 20 s
  later worked, and the error now has plain words.
- The open drawer covered the form; on wide windows it now pushes the page over.
- A test-script mistake sent one 19-second test video into the owner's real
  Downloads folder. The app correctly refused the bad folder path and fell
  back to the default. The file was moved out to the session scratchpad.

- **License → GPL-3.0-or-later** (owner: "lets make it gpl xD"; D-20) and
  **renamed to "Video Vacuum"** (owner: "yeah spell it right xD Video Vacuum"; D-21).
- **Installer packaging failed** at first with os error 4395 (reparse point):
  Claude's MSIX-redirected AppData held a stale July NSIS that Tauri couldn't
  delete. The stale folder was renamed (`tauri.old-2026-07` in Claude's
  package cache), and Tauri fetched NSIS 3.13 itself.

**Where we landed:** v2 merged into `main`; nothing pushed.

**Next:** the owner's walkthrough of the installed app; then merge, push and a GitHub Release.

### Session 2026-11-20 — vault notes
- Filed the reusable knowledge from v1 in the Obsidian vault and found the
  port-sharing bug while checking a claim for those notes.

### Session 2026-10-25 — v1: the local web edition
- Built v1: Python stdlib server on 127.0.0.1, a browser page and `Start.bat` /
  `start.sh` launchers; MP4/WebM/MKV/M4A; Update yt-dlp; localhost hardening.
  Restyled and renamed to "Video Saver". Pushed to the private repo.
- Tests (from the old `NOTES.md`):
  - link cleanup: 11 good forms + 9 bad inputs
  - security curl matrix
  - ffprobe on every format
  - Update via winget
  - `Start.bat` by full path
