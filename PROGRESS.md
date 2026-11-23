# Video Vacuum — Progress Log

Newest on top. Forward-looking items live in `ROADMAP.md`; the why lives in `docs/DECISIONS.md`.

### Session 2026-11-22 (night) — history privacy, MP3, the author name
**Goal:** owner: "there should be a clear button to just delete it all right
away. with an option to turn it off … And a way to delete individual
histories", "How hard is it to save the audio as .mp3 with a selectable
bitrate?", and the footer's "TheMysticTestudine" is the wrong account.

**What we did** (branch `feature/history-privacy-mp3`):
- **History → Settings** (was "Default folder"; D-28):
  - Keep download history on/off.
  - Clear history… with a confirm step.
  - Remove on every entry, including moved files.
  - Files are never deleted.
  - History off still lets that session's Open / Show in folder work.
- **MP3** format with a 128/192/256/320 kbps Bitrate menu, 192 default (D-27).
  The backend refuses any other bitrate.
- **TheMysticTurtle** in the window's legal notice, the installer copyright
  and the README (D-29). Git `user.name` is TheMysticTurtle in this repo and
  globally; a standing rule for every project is in the owner's global Claude
  instructions.
- Docs: README (MP3 row, History/Settings steps), PRIVACY (the new controls),
  TECH_REFERENCE (19 tests, LAME, test-app data folder), ROADMAP, D-27 to D-29.

**Test results:**
- clippy `-D warnings` clean, `cargo test` 19 passed (4 new: remove/clear,
  history off, old settings file, MP3 bitrate checks), tsc clean.
- Real window (hidden, CDP, its own `…devtest` data folder; "Me at the zoo"):
  - MP3 at 128, 192 (default, untouched menu) and 320: ffprobe showed mp3
    128000 / 192000 / 320000 bit/s.
  - M4A (AAC, unconverted), MP4 240p and WebM still saved.
  - Remove took one entry off and history.json matched.
  - History off: a WebM wasn't recorded; settings.json had `"keepHistory": false`;
    the "History is off" note showed.
  - Clear history: Cancel kept 4 entries; Clear emptied the list and the file;
    all 6 downloaded files were still there; the button greyed out when empty.
  - History back on: the next download was recorded.
  - Screenshots in light and dark; the footer reads "© 2026 TheMysticTurtle".
- The owner's real history.json and settings.json were byte-identical afterwards.

**Surprise:** stopping the test app with `taskkill /IM video-vacuum.exe` also
closed a second Video Vacuum process, which may have been the owner's
installed copy. TECH_REFERENCE §4 now says to stop the test app by PID.

**Where we landed:** merged into `main`, not pushed.

**Next:** the owner decides on releasing v2.1.0 (version bump + Release workflow).

### Session 2026-11-22 (late) — privacy, going public, docs refresh
**Goal:** a privacy statement, make the repo public without exposing the
owner's email, report SmartScreen's warning to Microsoft, and bring the docs
up to date after the owner's walkthrough.

**What we did:**
- Fixed the installer file name in the README and status docs (`0fa4bea`, `8881aa7`).
- Wrote `PRIVACY.md` from a code audit (collects nothing; what goes to
  YouTube, winget and WebView2; what's stored locally and how to delete it),
  plus a README Privacy section (`f7595a2`).
- Rewrote every commit and tag email to the GitHub noreply address, verified
  nothing else changed, force-pushed each ref with a lease, deleted the old
  Actions run, then made the repo public (D-26). Git on this PC now commits
  with the noreply address everywhere.
- SmartScreen's "report an incorrect warning" form, filled in for the
  installer's download link. The owner adds name, email and street address,
  solves the CAPTCHA and submits.
- The owner did the walkthrough of the released installer.
- Docs refresh:
  - ROADMAP brought up to date.
  - README: the History button and "Default folder" replace the old right-edge
    tab and ⚙ menu (D-24); "Paste link".
  - TECH_REFERENCE: 16 tests, installer size, free Actions minutes now that the repo is public.
  - CLAUDE.md status; D-26.
  - "Video Vacuum" titles on PROGRESS and DECISIONS.

**Test results:**
- The rewrite check passed on every ref: identical trees, dates, names,
  messages and parent counts, and no old email left.
- `PRIVACY.md` on GitHub returns HTTP 200.
- The v2.0.0 release and its installer are intact.
- Docs refresh (docs plus one code comment): clippy `-D warnings` clean, `cargo test` 16 passed, `tsc` clean.

**Where we landed:** v2.0.0 public, docs current.

**Next:** the owner submits the SmartScreen form and uploads the social preview
image; nothing else queued.

### Session 2026-11-22 (night) — GPT's restyle + brand art
**Goal:** finish GPT's "dark studio" restyle (it ran out of usage before
committing or launching), make it follow system light/dark, and wire in the
owner's art: icons, app header, installer, README/GitHub.

**What we did:**
- Found GPT's work in a separate clone, `worktrees/dark-studio` (7 files, uncommitted).
- Moved it into this repo on `feature/dark-studio` as GPT left it (`b6f33a7`),
  then committed Claude's light/dark change on top (`2a1a2dc`).
- Added `tools/make-brand-images.ps1` (built-in Windows imaging) and generated:
  - icons
  - header banner (406x88, 56 KB)
  - README banner (122 KB)
  - social preview (125 KB)
  - installer header and sidebar bitmaps
- Wired them into the masthead, NSIS (`installerIcon`, `headerImage`,
  `sidebarImage`) and the README. `media/` and `worktrees/` are git-ignored.
- D-24 records the look and the art.

**Test results:**
- clippy clean, 16 tests passed, tsc clean.
- Installer built (2.0 MB). Silent install exit 0. The installed exe shows the
  new logo icon.
- The installed app's header banner loaded (406 px source, shown 44 px); light
  and dark screenshots look right. Uninstall exit 0.

**Surprises:**
- The banner's glow background is stored in fully transparent pixels, so
  there are two possible looks, and the owner picks.
- The art is ~94-99% opaque, which was measured and is fine.
- .NET's icon reader can't decode Tauri's PNG-in-ICO; extracting the PNGs
  showed the icons are fine.

- Owner picked the transparent banner ("no glow background"). The hero now
  shows the Alt banner with "New download" underneath, replacing GPT's film
  reel (D-25). Previewed at 1000x740 in light and dark, and at 720x600.

- **Pushed and released:** the owner said "push it and start the
  auto-release-build pipeline". Release run 45001774325 passed every step
  (version guard, tsc, clippy, 16 tests, build, release) in 20 min.
  Published [v2.0.0](https://github.com/TheMysticTurtle/video-vacuum/releases/tag/v2.0.0),
  tag at `43db3d8`, installer `Video.Vacuum_2.0.0_x64-setup.exe` (2.56 MB;
  GitHub turned the space into a dot, so the README now uses that name).

**Where we landed:** `main` merged, pushed and released as v2.0.0.

**Next:** the owner tries the released installer; decide when the repo goes public.

### Session 2026-11-22 (later) — efficiency pass
**Goal:** owner: "make sure it's as efficient as it can be. not re-reading stuff it doesnt need to or duplicating calculations/data flows." Approved: "Fix all seven".

**What we did:** branch `perf/efficiency`, D-23:
- downloads reuse the paste-time lookup (`--load-info-json`)
- the page sends only its choices
- right-sized thumbnails
- tools found at startup (fixes a startup race)
- a single tool scan
- history saved without copying
- the download folder read once
- README and release notes: "Requires Windows 10 or 11".

**Test results:**
- `cargo test` 16 passed (new tests: lookup reuse, stale lookup falls back to
  the link, the temp file is removed, thumbnail pick); clippy `-D warnings`
  clean; `tsc` clean.
- **Measured first** (plain yt-dlp, 3 runs each): download from the link
  avg 3.91 s vs from the saved lookup avg 1.75 s; identical output (ffprobe
  h264+aac 144p). `yt-dlp --version` 0.93 s. Thumbnails: maxres WebP 132 KB vs
  hqdefault 31 KB.
- **Real window afterwards** (hidden, CDP):
  - MP4 240p saved in 1.7 s on 3 runs (was 3.5 s); M4A 2.3 s (was 4.1 s);
    WebM 1.7 s (was 3.5 s). All verified with ffprobe.
  - Names: "(2)"/"(3)" only for the same type.
  - Cancel at 4K: nothing kept, no processes, no leftover lookup files.
  - History entries carry title, channel and thumbnail from the lookup.
  - Card thumbnail 480 px.
  - Fresh start, paste 0.27 s after load (before the version check
    returned): lookup worked.

**Where we landed:** merged into `main`, not pushed.

**Next:** push, then the first Release run (v2.0.0).

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

- **GitHub repo renamed** to `video-vacuum` (D-22). **Pushed** `main` and tag `v1.0.0-local-web`.
- **Release workflow** added: `.github/workflows/release.yml`, manual only. It
  runs the version guard → quality gate → `tauri-action` build → release
  (TECH_REFERENCE §6). Actions are pinned to SHAs. Found and fixed before
  shipping: the first guard used `git ls-remote`, which would silently fail
  against the private repo once the checkout drops its credentials. It now
  uses the GitHub API and stops on any error except "Not Found". Guard tested
  locally: new version passes, existing tag stops it, bad token stops it.

**Where we landed:** v2 + the release workflow on `main`, pushed.

**Next:** the first Release run (makes GitHub release v2.0.0) and the owner's walkthrough of the installed app.

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
