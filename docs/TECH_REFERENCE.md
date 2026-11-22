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

**Outside programs the app runs (not shipped, installed separately; D-15):**
- yt-dlp: Unlicense, https://github.com/yt-dlp/yt-dlp. winget `yt-dlp.yt-dlp`.
- FFmpeg: GPL build, winget `yt-dlp.FFmpeg`.
- Deno: MIT, winget `DenoLand.Deno`.

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
npm run tauri dev      # development window with hot reload
npm run tauri build    # src-tauri/target/release/bundle/nsis/Video Vacuum_<version>_x64-setup.exe
```

- Release profile: LTO, `opt-level = "s"`, stripped. The exe is about 7 MB; the installer about 1.7 MB.
- The installer is NSIS, per user (`installMode: currentUser`), no admin prompt. WebView2 uses Tauri's default `downloadBootstrapper`.

## 3. Quality gate

```sh
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
npm run check   # tsc --noEmit
```

The Rust tests (14) cover:
- link cleanup (12 good forms, 9 bad)
- file-name safety
- progress-line parsing
- plain-word errors
- yt-dlp JSON → card info
- format options
- unique naming and cleanup that only touches its own files
- history/settings save, reload and newer-file protection
- PATH variable expansion
- winget detection

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
- The dev app uses the real app-data folder. Put test downloads in a scratch folder and clean up afterwards.

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
- **`std::fs::rename` replaces an existing file on Windows**, which makes the temp-then-rename save atomic.
- **WebView2 `prefers-color-scheme`** follows the Windows app theme, so light/dark needs no code.

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
- **Cost:** private repos pay double for Windows minutes. It's one job, and Rust builds are cached.
