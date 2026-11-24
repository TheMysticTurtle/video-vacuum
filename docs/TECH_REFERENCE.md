# Video Vacuum — Tech Reference

Stack, versions, licenses, official sources, how to build and test, and the
gotchas hit along the way. Versions are as of **2026-11-22**.

## 1. Stack

| Part | Version | License | Official source | Used for |
|---|---|---|---|---|
| Tauri (`tauri`, `tauri-build`, `@tauri-apps/cli`, `@tauri-apps/api`) | 2.12 | MIT OR Apache-2.0 | https://v2.tauri.app · crates.io / npm `@tauri-apps` | App shell, window, IPC, installer bundling |
| `tauri-plugin-dialog` / `@tauri-apps/plugin-dialog` | 2.8 | MIT OR Apache-2.0 | https://v2.tauri.app/plugin/dialog/ | Folder picker |
| `tauri-plugin-opener` | 2.7 | MIT OR Apache-2.0 | https://v2.tauri.app/plugin/opener/ | Open file / show in folder / open YouTube link (called from Rust only) |
| `serde`, `serde_json` | 1 | MIT OR Apache-2.0 | crates.io | JSON |
| `url` | 2 | MIT OR Apache-2.0 | crates.io (servo/rust-url) | Parsing pasted links |
| `winreg` (Windows only) | 0.56 | MIT | crates.io | Reading the current PATH from the registry |
| React, React DOM | 19.3 | MIT | npm `react` | UI |
| Vite + `@vitejs/plugin-react` | 8.3 / 6.3 | MIT | npm | Dev server and build |
| TypeScript | **5.9** (pinned) | Apache-2.0 | npm | Type checking. 7.x is the new native compiler; held at 5.x per the vault note "React and TypeScript - Toolchain Gotchas" |
| Rust | 1.97 (plugins need ≥ 1.90) | — | rustup | Backend |
| WebView2 | runtime 157 on the dev PC | — | Built into Windows 10 (1803+) / 11 | Renders the window |
| NSIS (fetched by Tauri) | 3.13 + `nsis_tauri_utils` 0.5.3 | zlib/libpng | github.com/tauri-apps/binary-releases | Building the installer (build tool, not shipped) |

**Bundled tools (shipped in `<install dir>\tools\`; D-32).** Pinned in
`src-tauri/tools.json`, with each version, official download and SHA-256:

| Tool | Version | License | Official source | Used for |
|---|---|---|---|---|
| yt-dlp (`yt-dlp.exe`) | 2026.11.23 | Unlicense (bundled third-party notices in `yt-dlp-THIRD_PARTY_LICENSES.txt`) | github.com/yt-dlp/yt-dlp releases | Downloading |
| Deno (`deno.exe`) | 2.9.8 | MIT | github.com/denoland/deno releases (the full `deno` zip, not `denort`) | YouTube's JS challenges (yt-dlp's recommended runtime; minimum 2.3.0) |
| FFmpeg (`ffmpeg.exe`, `ffprobe.exe`) | 9.1 + LAME 3.100 | LGPL-2.1-or-later (no `--enable-gpl`) | Built by `.github/workflows/build-ffmpeg.yml` from ffmpeg.org (GPG-verified) and SourceForge LAME (SHA-256 pinned); published as the `tools-ffmpeg-9.1` release | Merging, repackaging, MP3 |

- **Trimmed FFmpeg:** `--disable-everything` plus only these:
  - the demuxers mov, matroska, mp3, aac, ogg, mpegts, wav and flac
  - the muxers mp4, ipod, mov, matroska, webm, mp3, adts, ogg and opus
  - the matching parsers and bitstream filters
  - the decoders aac, opus, vorbis, mp3, flac and pcm
  - `libmp3lame`
  - the filters aresample, aformat, anull, atrim, null, format, trim and copy
  - no network

  Result: 4.9 MB + 4.6 MB, against about 160 MB each for full builds.
  Spike run 45319314183 passed all 10 format tests.
- **License texts** ship in `tools\licenses\`:
  - `src-tauri/tools-licenses/` (yt-dlp, its third-party notices, Deno)
  - the FFmpeg zip's `FFmpeg-LICENSE.md`, `COPYING.LGPLv2.1.txt`,
    `LAME-COPYING.txt` and `SOURCES.txt`
- **To change a tool's version:**
  1. Update `tools.json` (version, URL, SHA-256).
  2. For FFmpeg, run Build FFmpeg, test the zip, publish it as a new
     `tools-ffmpeg-<v>` release, then pin it.
  3. Re-run `tools/fetch-tools.ps1`.
- **Windows' own `curl.exe` and `tar.exe`** (System32, Windows 10 1803+/11)
  do Restore's download and unzip. They're called by full path. `sha2`
  (RustCrypto, MIT OR Apache-2.0) checks the checksum.

### License scan (2026-11-22)

`cargo metadata --filter-platform x86_64-pc-windows-msvc` found 251 crates in
the Windows build. The licenses were MIT / Apache-2.0 (in their various
spellings), Unicode-3.0, Unlicense, BSD-3-Clause, Zlib, CC0 and **MPL-2.0** (5:
`cssparser`, `cssparser-macros`, `dtoa-short`, `selectors`, `option-ext`, from
Tauri's dependency tree). None were GPL, AGPL or unlicensed. The npm packages
shipped in the UI are 5, all MIT or MIT/Apache-2.0. All of these are compatible
with the app's own GPL-3.0-or-later (D-20).

## 2. Build

```sh
npm install
powershell -ExecutionPolicy Bypass -File tools/fetch-tools.ps1   # bundled tools -> src-tauri/tools/ (checksum-checked)
npm run tauri dev      # development window with hot reload
npm run tauri build    # src-tauri/target/release/bundle/nsis/Video Vacuum_<version>_x64-setup.exe
```

- The Rust build bundles `src-tauri/tools/` (a Tauri resource), so it fails
  with "resource path `tools` doesn't exist" until the fetch script has run.
- Before the FFmpeg zip is published: `tools/fetch-tools.ps1 -FfmpegZip <zip>`.

- Release profile: LTO, `opt-level = "s"`, stripped. The exe is about 7 MB; the installer about 2.1 MB built locally, 2.56 MB from the Release workflow (v2.0.0; the brand art added ~0.4 MB).
- The installer is NSIS, per user (`installMode: currentUser`), no admin prompt. WebView2 uses Tauri's default `downloadBootstrapper`.

## 3. Quality gate

```sh
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
npm run check   # tsc --noEmit
```

The Rust tests (25) cover:
- link cleanup (12 good forms, 9 bad)
- file-name safety
- progress-line parsing
- plain-word errors
- yt-dlp JSON → card info
- format options, including MP3 bitrates and refusing a bitrate not on the list
- unique naming and cleanup that only touches its own files
- history/settings save, reload and newer-file protection
- removing an entry, clearing history, history turned off (nothing written,
  but the session can still open the file), v2.0.0 settings reading as "keep history"
  and "system" theme, the theme saved and read back
- the diagnostic log: UTC dates (round trip, leap day), the file line format,
  memory-only until saving is on, 30-day cleanup that touches only our files,
  the 2000-line cap, skipping exact repeats; yt-dlp warnings and step names
- bundled tools: `tools.json` is complete, versions parsed from each tool's
  output, numeric version comparison, SHA-256 enforcement, a missing folder
  reports every tool missing, yt-dlp gets the bundled paths, files found in
  zip subfolders

## 4. Testing the real window (hidden, off the owner's screen)

WebView2 accepts Chromium flags from an environment variable, so the dev
window can be driven over the Chrome DevTools Protocol:

```sh
WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS="--remote-debugging-port=9223" \
  npx tauri dev --config '{"app":{"windows":[{"label":"main","title":"Video Vacuum","width":1000,"height":740,"visible":false}]}}'
```

- `--config` merges into `tauri.conf.json`, but **arrays are replaced whole**, so give the full window object.
- A hidden window still renders: CDP `Page.captureScreenshot` returns the real page.
- A small Node script (Node 24 has `WebSocket` built in) talks to the debug
  port. It uses `Runtime.evaluate` to paste (a `ClipboardEvent` with a
  `DataTransfer`), set `<select>` values (via the native value setter, then a
  `change` event) and click buttons. It can also call Rust commands directly
  through `window.__TAURI_INTERNALS__.invoke`.
- **Give the test app its own data folder** by adding `"identifier":"com.themysticturtle.videovacuum.devtest"`
  to the `--config` JSON. Otherwise the dev app uses the owner's real history
  and settings, and a Clear history test would wipe them. Then set the default
  folder to a scratch folder (`invoke('set_default_folder', …)`), and delete
  both `…devtest` folders in `%APPDATA%` and `%LOCALAPPDATA%` afterwards.
- **Stop only the test app's own processes** (by PID, or `TaskStop` on the dev
  command plus the Vite server on port 1420). `taskkill /IM video-vacuum.exe`
  also closes the owner's installed copy if it's open.

## 5. Gotchas

- **Builds started from the Claude desktop app see a redirected AppData**
  (MSIX). Tauri's NSIS cache existed twice: the real
  `%LOCALAPPDATA%\tauri` and `…\Packages\Claude_…\LocalCache\Local\tauri`. When
  Tauri 2.12 decided the July NSIS was outdated, deleting it through the merged
  view failed with *os error 4395 (reparse point)*. Fix used 2026-11-22: rename
  the stale redirected folder; Tauri then fetched NSIS 3.13 itself. Installs
  started from Claude also land in the redirected AppData, so **the owner's
  double-click is the real install test.** (Vault: "Python - Release Build
  Pitfalls on Windows".)
- **Cancel must end the process tree** (`taskkill /T /F`); yt-dlp starts ffmpeg (D-17).
- **yt-dlp output is read as bytes** and decoded lossily per line; `--encoding utf-8` keeps titles intact.
- **YouTube sometimes answers rapid repeated downloads with HTTP 403.** It's temporary; a retry ~20 s later worked. The app explains it in plain words.
- **Don't let yt-dlp look a video up twice.** The paste-time `-J` JSON is
  reused for the download with `--load-info-json <file>` (1.75 s vs 3.9 s, same
  file). Its stream links expire after a few hours, so lookups older than
  1 hour are redone from the link (D-23).
- **Startup:** `Tools::find()` runs in `setup` (registry PATH read and file checks only).
  The `yt-dlp --version` call (~0.9 s, a separate process) runs afterwards and
  never gates lookups.
- **`std::fs::rename` replaces an existing file on Windows**, which makes the temp-then-rename save atomic.
- **WebView2 `prefers-color-scheme`** follows the Windows app theme, so "System" needs no code.
- **Theme (D-30):**
  - Colors use CSS `light-dark()` plus `color-scheme`. Chromium supports it
    from version 123; WebView2 is evergreen and was runtime 157 on the dev PC.
  - `data-theme="light|dark"` on `<html>` forces a theme.
  - The title bar follows `Window::set_theme`, called from Rust, so the page
    needs no `core:window:allow-set-theme` permission.
- **Window size (D-30):** `tauri.conf.json` asks for 1000 x 960 (measured: a
  looked-up video plus every save option, no scrolling). `fit_window_to_screen`
  in `lib.rs` shrinks it to `Monitor::work_area` on smaller screens. Tested
  with a 3000 px tall window: it came out exactly the monitor's usable height.

## 6. Releasing (GitHub Actions)

`.github/workflows/release.yml` runs **only when started by hand**: Actions tab
→ **Release** → **Run workflow**, or:

```sh
gh workflow run release.yml                 # publish right away
gh workflow run release.yml -f draft=true   # save as a draft to review first
```

1. **Version guard:** `tauri.conf.json`, `package.json` and `Cargo.toml` must
   carry the same version, and `v<version>` must not exist yet. The check uses
   the GitHub API with the read token, because the checkout keeps no
   credentials. Only "Not Found" passes; any other error stops the run.
   Tested locally: a new version passes, an existing tag stops it, a bad token stops it.
2. **Quality gate:** `npm ci` → `npm run check` → `cargo clippy --all-targets -D warnings` → `cargo test`.
3. **Build and release:** `tauri-apps/tauri-action` runs `tauri build`
   (NSIS), creates the tag `v<version>` on the run's commit, and creates the
   release "Video Vacuum v<version>" with the installer and install notes.

- **To release a new version:** bump the version in all three files, commit, push, run the workflow.
- **Security** (GitHub's "Security hardening for GitHub Actions"):
  - every action is pinned to a full commit SHA, with the release in a comment
  - the workflow's token is read-only, and only this job gets `contents: write`
  - `persist-credentials: false` on checkout
- **Pinned actions (2026-11-22):**
  - `actions/checkout` v7.0.1
  - `actions/setup-node` v7.1.0
  - `dtolnay/rust-toolchain` stable
  - `Swatinem/rust-cache` v2.9.3
  - `tauri-apps/tauri-action` action-v1.0.0

  To update one, look up the new release's commit SHA (`gh api repos/<owner>/<repo>/commits/<tag> --jq .sha`).
- **Cost:** free. The repo is public (D-26), and GitHub-hosted runners cost nothing on public repos. While it was private, Windows minutes counted double.
- **First run (v2.0.0, 2026-11-22):** 20 minutes, mostly compiling Rust with a
  cold cache; later runs reuse the cache.
- **GitHub renames assets with spaces:** `Video Vacuum_2.0.0_x64-setup.exe` is
  published as `Video.Vacuum_2.0.0_x64-setup.exe`. Refer to the dotted name in docs.

## 7. Brand images and screenshots

**README screenshots** (`docs/images/screenshots/*.png`) are the owner's own
demo captures, copied by hand from `media/` (which git ignores):

| File | From `media/` |
|---|---|
| `start.png` | `Starting-Screen-Demo.png` |
| `video-loaded.png` | `Video-Loaded-Demo.png` |
| `history.png` | `History-Demo.png` |
| `settings.png` | `Settings-Demo.png` |
| `log.png` | `Log-Demo.png` |

Retake them when the window changes noticeably.

**Brand images:**

Originals live in `media/` (ignored by git). Every small copy is made by
`tools/make-brand-images.ps1`, which uses only Windows' built-in System.Drawing:

```powershell
powershell -ExecutionPolicy Bypass -File tools\make-brand-images.ps1   # add -BannerGlow to keep the banner's glow background
npx tauri icon src-tauri/icons/app-icon.png -o src-tauri/icons          # then delete the android/ ios/ Square*/StoreLogo/64x64 extras
```

| Output | From | Used by |
|---|---|---|
| `src-tauri/icons/*` (16-256 px .ico, .png, .icns) | LogoLarge-Transparent → 1024 px → `tauri icon` | window, taskbar, Start Menu, exe, installer icon |
| `src/assets/header-banner.png` (406x88) | SkinnyBanner, trimmed to its artwork | masthead (shown 44 px tall) |
| `docs/images/banner.jpg` (1280x480) | Banner | README |
| `docs/images/social-preview.jpg` (1280x640) | Banner, top/bottom edges extended | GitHub Settings → Social preview (upload by hand; no API) |
| `src-tauri/installer/header.bmp` (150x57) | Banner | NSIS `headerImage` |
| `src-tauri/installer/sidebar.bmp` (164x314) | LogoLarge (dark tile) | NSIS `sidebarImage` |

Gotchas:
- **The art is AI-cut:** its "transparent" pixels still hold colors, and the
  artwork is ~94-99% opaque rather than solid. Measured; it doesn't wash out.
- **Old .NET's `Icon.ToBitmap` can't decode the PNG-compressed entries Tauri
  writes into `.ico`.** It shows noise. To inspect an icon, read the ICO
  directory and extract the embedded PNGs instead.
