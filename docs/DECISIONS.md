# Video Vacuum — Decisions

Every locked choice and why. A revisit is a new entry that supersedes the old
one; entries are never silently edited. D-01 to D-10 come from v1 (the local
web edition, tag `v1.0.0-local-web`).

### D-01 ✅ Runs on the user's own computer, never a hosted service (2026-10-25)
**Decision:** each user runs the app locally; nothing is hosted.
**Why:** a static page can't run yt-dlp, and a public download service is a
ToS/legal grey area that YouTube also blocks.
**Rejected:** a GitHub Pages PWA backed by a server.

### D-02 ✅ Drive the `yt-dlp` program, not its Python module (2026-10-25)
**Decision:** run the installed `yt-dlp` executable with an argument list.
**Why:** stays independent of how yt-dlp was installed, and winget / `-U` can update it.

### D-03 ✅ Predictable yt-dlp options (2026-10-25)
**Decision:** always pass `--ignore-config --no-playlist --encoding utf-8`, and `--` before the URL.
**Why:** the user's yt-dlp config can't change results; a `&list=` link means
just that video; emoji titles survive the pipe; nothing after `--` is read as an option.

### D-04 ✅ Quality = `-S res:N`; MP4 = `codec:h264:m4a` (2026-10-25)
**Decision:** cap resolution with `-S res:N` (README, "Sorting Formats"). MP4
sorts `codec:h264:m4a`.
**Why:** sorting by `ext` alone produced VP9 in an `.mp4` (ffprobe, 144p).
H.264 + AAC is what "plays everywhere" means. YouTube only offers H.264 up to 1080p.

### D-05 ✅ No re-encoding (2026-10-25)
**Decision:** ffmpeg only joins/remuxes streams. M4A takes YouTube's AAC as-is.
**Why:** owner, 2026-10-25: "just download what youtube sends ya xD and adjust the quality settings accordingly."

### D-06 ✅ Progress through `--progress-template` markers + JSON (2026-10-25)
**Decision:** `@@P {json} vcodec`, `@@S postprocessor`, `@@F filepath` lines.
**Why:** reliable parsing instead of scraping the human-readable progress bar.

### D-07 ✅ Update through whatever installed yt-dlp (2026-10-25)
**Decision:** `winget upgrade --id yt-dlp.yt-dlp` for winget installs, `yt-dlp -U` otherwise.
**Why:** yt-dlp README: package-manager installs update through that manager.

### D-08 ✅ Never accept winget agreements for the user (2026-10-25)
**Decision:** winget installs run in a visible console without `--accept-*` flags.
**Why:** the user answers winget's own prompts.

### D-09 ✅ Name "Video Saver", calm look, no platform red (2026-10-25)
**Why:** owner, 2026-10-25: "this looks like the most virus ass thing i've ever
seen in my life xD". The YouTube name and red read as a clone site and borrow
their branding. See the vault lesson "Don't Borrow a Platform's Branding".

### D-10 ✅ (v1 only) Exclusive port bind on Windows (2026-11-21)
**Decision:** v1's server set `SO_EXCLUSIVEADDRUSE`, so a second copy moves to the next port.
**Consequence:** no longer relevant from v2 (no local server).

### D-11 ✅ A desktop app, not a PWA or browser extension (2026-11-21)
**Decision:** rebuild Video Saver as an installable desktop app.
**Why:** owner, 2026-11-21: "lets stamp what we have as the old local PWA
version. And make it a proper little app." Neither a PWA nor an extension can
run yt-dlp; the Chrome Web Store prohibits products that enable unauthorized
downloading of streaming content. The modern yt-dlp GUIs (Open Video Downloader
v3, NeoDLP) are Tauri desktop apps.
**Rejected:** PWA (still needs a local server), browser extension (needs a companion app anyway).
**Supersedes:** v1's local web architecture (kept as tag `v1.0.0-local-web`).

### D-12 ✅ Tauri 2 + React + TypeScript (2026-11-21)
**Decision:** a Rust backend in a Tauri 2 shell, React 19 + TypeScript UI, built with Vite.
**Why:** the same family as Palbox Studio; Tauri builds the installer, icon and
Start Menu entry itself; no Python for users to install; no server, browser tab
or console window. The owner leaned React ("Can it be as simple as a react app?").
**Rejected:** pywebview + PyInstaller (packaging pain, antivirus false
positives), Electron (heavy), Qt (heavy, LGPL duties).

### D-13 ✅ The page never runs programs or opens files itself (2026-11-21)
**Decision:** all actions go through validated Rust commands. Links are rebuilt
from a video ID; files open only if the app downloaded them (looked up by
history ID); `open_link` accepts only https YouTube links. The window's
capability allows only `core:default` and `dialog:allow-open`. Strict CSP: self
plus `*.ytimg.com` images.
**Why:** least privilege, the same idea as v1's localhost hardening.

### D-14 ✅ Progress over a Tauri Channel (2026-11-21)
**Decision:** `start_download` takes a `Channel<DownloadEvent>`.
**Why:** Tauri docs ("Calling the Frontend") recommend channels for streaming
such as download progress; events are not designed for high throughput.

### D-15 ✅ yt-dlp, ffmpeg and deno stay separate installs (2026-11-21)
**Decision:** the app finds them (process PATH, the registry PATH and winget's
Links folder) and offers "Install missing tools" through winget. The installer
doesn't bundle them.
**Why:** keeps the installer small; the Update button keeps working; avoids
redistributing yt-dlp's GPL FFmpeg build. Reading the registry PATH means
tools installed while the app is open are found without a restart.

### D-16 ✅ Never overwrite; cleanup only removes what this download created (2026-11-21)
**Decision:** a name is taken when a file of the *same type* exists (or a
`.part`/`.ytdl` from a download in progress). Then "name (2)", "(3)"…. Before
starting, the app snapshots files already named "name.*"; after a cancel or
failure it deletes only new ones. `--no-overwrites` as a second guard.
**Why:** the first version counted any extension, so an `.m4a` pushed a new MP4
to "(3)" (seen in testing).

### D-17 ✅ Cancel ends the whole process tree (2026-11-21)
**Decision:** `taskkill /PID <pid> /T /F` on Windows.
**Why:** killing only yt-dlp would leave its ffmpeg running. Tested: cancel at
4% of a 4K download left no processes and no partial files.

### D-18 ✅ History and settings as two small versioned JSON files (2026-11-21)
**Decision:** `history.json` and `settings.json` in the app data folder
(`%APPDATA%\com.themysticturtle.videosaver`). Saves are atomic (temp file +
rename). A file from a newer version, or a damaged one, is never overwritten.
**Why:** tiny data with no need for a database; the robustness rules from the
vault note "Desktop App Robustness Patterns".

### D-19 ✅ Clips, preview and GIF export are deferred, not designed out (2026-11-21)
**Decision:** v2 ships the plain downloader with a thumbnail. Owner,
2026-11-21: "make the basic downloader app as it is without the gif download …
Lets make sure we don't close that door on ourselves."
**Consequence:** the thumbnail spot in `VideoCard` is where a preview would go;
`format_args` in `ytdlp.rs` is the one place a `--download-sections` option
would be added. See ROADMAP backlog for the research already done.

### D-20 ✅ Video Saver is GPL-3.0-or-later (2026-11-22)
**Decision:** relicense from MIT to the GNU GPL, version 3 or (at your option)
any later version. The official text from gnu.org is in `LICENSE`.
**Why:** owner, 2026-11-22: "oh. i dont give a shit. lets make it gpl xD". With
the GPL, anyone who ships a changed version has to keep it open too. "Or later"
is the FSF's recommended form. v3 (not v2) because GPLv2 isn't compatible with
Apache-2.0 code, and Video Saver uses Apache-2.0 crates.
**Consequence:**
- Every dependency must be GPLv3-compatible. The 2026-11-22 scan found only
  MIT / Apache-2.0 / BSD / Zlib / Unicode / Unlicense / CC0 / MPL-2.0, so
  all of them are fine.
- Versions up to tag `v1.0.0-local-web` stay MIT; they were released that way.
- The window shows a short free-software / no-warranty notice
  ("Appropriate Legal Notices", GPLv3 §0 and §5d).
- yt-dlp, FFmpeg and Deno remain separate programs that the app runs, not part of it.
**Supersedes:** the MIT license.

### D-21 ✅ Renamed to "Video Vacuum" (2026-11-22)
**Decision:** the app is called **Video Vacuum** from v2. App ID
`com.themysticturtle.videovacuum`, so its data folder is
`%APPDATA%\com.themysticturtle.videovacuum`; packages `video-vacuum`; the
installer is `Video Vacuum_<version>_x64-setup.exe`.
**Why:** owner, 2026-11-22: "I think i'd like to call it Video Vaccuum" →
"yeah spell it right xD Video Vacuum". It's still a name of its own, not
YouTube's (D-09 holds).
**Consequence:**
- Safe to change the app ID now, because nobody has installed v2 yet.
- The GitHub repo is still `TheMysticTurtle/video-saver` and the project
  folder is still `YoutubeVideoDownloader`; renaming them is the owner's call.
- Entries above D-21 keep the old name as history; D-18's data folder is now
  the `videovacuum` one.
**Supersedes:** the name part of D-09.

### D-22 ✅ GitHub repo renamed to `video-vacuum` (2026-11-22)
**Decision:** `TheMysticTurtle/video-saver` → `TheMysticTurtle/video-vacuum`
(still private). The local `origin` remote points to the new URL.
**Why:** owner, 2026-11-22: "can we re-name the repo to Video Vacuum?" GitHub
names can't contain spaces, so it's the dash form.
**Consequence:** GitHub redirects the old URL. The local project folder is still `YoutubeVideoDownloader`.
**Supersedes:** the repo-name note in D-21.

### D-23 ✅ Efficiency pass: no repeated work (2026-11-22)
**Decision:** owner, 2026-11-22: "make sure it's as efficient as it can be. not
re-reading stuff it doesnt need to or duplicating calculations/data flows."
Seven changes, each measured or tested:
1. **Downloads reuse the lookup.** The `-J` JSON from pasting is written to a
   temp file and passed as `--load-info-json`. A lookup older than 1 hour is
   redone from the link, because YouTube's stream links expire after a few
   hours. The temp file is removed when the download ends. Measured: MP4 240p
   3.5 s → **1.7 s**, M4A 4.1 s → **2.3 s**, WebM 3.5 s → **1.7 s** (real
   window, same video). Plain yt-dlp: 3.9 s → 1.75 s.
2. **No video round trip.** The page sends only `{videoId, format, height,
   folder, fileName}`. Rust keeps the lookup and builds history from it. A
   mismatched or missing lookup is refused ("Look up the video again").
3. **Right-sized thumbnails:** the smallest ≥ 400 px wide (sharp at 200 px on
   high-DPI screens). Measured on a 4K video: 132 KB WebP → 31 KB JPEG.
4. **Tools are found at startup** before the window opens (no program
   started). The ~1 s `--version` check no longer gates lookups. Tested:
   pasting 0.27 s after load, before the check returned, worked; before this
   change it failed with "yt-dlp isn't installed."
5. **One folder scan** finds yt-dlp, ffmpeg, deno and winget.
6. **History saves borrow the list** instead of copying it.
7. **The download folder is read once** for name picking and the cleanup snapshot.
**Also:** the footer shows "…" (not "not found") until the version check returns.

### D-24 ✅ "Dark studio" look + the owner's brand art (2026-11-22)
**Decision:**
- Adopt GPT's restyle: masthead, "New download" hero with film-reel art, a
  header History button, and an "Engine ready" light. It follows the Windows
  light/dark setting (owner: "maybe we should let it follow system
  light/dark").
- Use the owner's art from `media/` everywhere: app icons from
  `VideoVacuumLogoLarge-Transparent.png`, the header from
  `VideoVacuum-SkinnyBanner.png`, and the README / social preview / installer
  header from `VideoVacuumBanner.png` (owner: "just banner is the banner we
  use. the ALT is stored as an alternative"). The installer sidebar uses
  `VideoVacuumLogoLarge.png`.
**How:** `tools/make-brand-images.ps1` (built-in Windows imaging, nothing to
install) makes every small copy from the originals. The originals stay out of
git (`media/` is ignored, per the software standard's "no large media").
Rerun the script, then `npx tauri icon`, whenever the art changes.
**Consequence:**
- The look is no longer "calm neutral" (D-09's look part is superseded by the
  owner's own brand). D-09's no-platform-branding rule still holds: it's the
  project's own art, not YouTube's.
- GPT's version removed the trust line ("Runs on your computer…") and the
  right-edge History tab. Both are kept out unless the owner wants them back.
- The 16 px icon is a blur of the detailed logo; a simplified small icon is an option.
- The banner's glow background is stored in fully transparent pixels. The
  default output shows the logo on navy, and `-BannerGlow` keeps the glow. The
  owner picks.
**Supersedes:** the look in D-09.

### D-25 ✅ Banner stays transparent; the hero is the Alt banner (2026-11-22)
**Decision:**
- The banner keeps its transparent background, with no glow (owner: "no
  glow background lol"). The default output of `make-brand-images.ps1` stays.
- The "New download" hero shows the owner's Alt banner
  (`VideoVacuumBanner-Alt.png` → `src/assets/hero-banner.png`, trimmed, 320 px
  tall) instead of GPT's CSS film reel. "New download" and the subtitle sit
  underneath it, on a darker radial stage so the neon art pops in both themes.
  Owner: "change that new download with the generic looking film thing to the
  ALT banner and new download worked in somewhere there with it. since it
  disappears when you type a link."
**Consequence:**
- The film-reel CSS is deleted.
- The hero still hides once a video is loaded.
- The hero image is ~516 KB; detailed glow art compresses poorly as PNG. It's
  local, with no network cost.

### D-26 ✅ Public repo, a privacy statement, and noreply commit emails (2026-11-22)
**Decision:**
- `PRIVACY.md` states what the app does and doesn't send (it collects nothing),
  and the README links it. Owner: "this thing doesnt collect any info right? do
  we have a privacy statement?"
- The GitHub repo is **public**. Owner: "nah i want the repo to be public xD".
- Before it went public, every commit and tag was rewritten to use the GitHub
  noreply address (`101545919+TheMysticTurtle@users.noreply.github.com`)
  instead of the owner's personal email. Owner: "swap to githubs private
  noreply address. i dont want weirdos pulling my email xD". Future commits
  use it too: it's set in this repo's git config and globally on the dev PC.
**How:** email-only rewrite with `git fast-export | swap script | git
fast-import`, after a backup bundle (kept outside the repo). A check proved
every ref's trees, dates, names, messages and parent counts were unchanged;
then each ref was force-pushed with a lease on its old value. The v2.0.0
release stayed attached to its tag. The old Actions run that pointed at the
pre-rewrite commits was deleted before the repo went public.
**Consequence:**
- Every commit ID changed on 2026-11-22. Clones made before then need a fresh clone.
- A public link was needed for Microsoft's SmartScreen "incorrect warning"
  form; `PRIVACY.md` on GitHub is that link.
**Supersedes:** "still private" in D-22.
