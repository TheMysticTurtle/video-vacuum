<h1 align="center"><img src="docs/images/banner.jpg" alt="Video Vacuum" width="720"></h1>

A small Windows app for saving YouTube videos. Paste a link, pick a format,
quality, name and folder, click **Download**.

It runs on your computer using [yt-dlp](https://github.com/yt-dlp/yt-dlp).
Nothing is uploaded anywhere.

## Install

**Requires Windows 10 or 11** (64-bit). Windows 7 and 8.1 aren't supported:
the window's browser engine (Microsoft WebView2), current Rust programs, and
the tools it uses all need Windows 10 or newer.

1. Download `Video.Vacuum_<version>_x64-setup.exe` (for example
   `Video.Vacuum_2.0.0_x64-setup.exe`) from the
   [Releases](https://github.com/TheMysticTurtle/video-vacuum/releases) page and run it.
   - It installs just for you (no admin prompt) and adds Video Vacuum to the Start Menu.
   - The installer isn't code-signed yet, so Windows may show "Windows protected
     your PC". Click **More info → Run anyway**.
2. Open **Video Vacuum**. The first time, if the free tools it needs are missing
   (yt-dlp, ffmpeg, deno), click **Install missing tools**. A winget window opens;
   answer its questions there.

## Use it

1. **Paste a YouTube link** (or click **Paste link**). Messy links are fine: tracking
   bits like `?si=` and `&t=` are trimmed, and Shorts, `youtu.be` and mobile links work.
2. Check the video: thumbnail, title, creator, upload date, length, views and
   the description. Click the title or creator to open them on YouTube.
3. Pick a **Format**, **Quality**, **File name** and **Folder**, then **Download**.
   You can cancel at any time; nothing half-finished is left behind.
4. **History** (the button at the top) opens a panel listing everything you've
   downloaded, with a search box. Files you've moved or deleted are greyed out.
   Click one to see where it used to be. **Default folder** in that panel sets
   where downloads go.

| Format | What you get |
|---|---|
| **MP4** | Most compatible: H.264 video + AAC audio up to 1080p (above 1080p YouTube only offers VP9/AV1) |
| **WebM** | VP9/AV1 video + Opus audio |
| **MKV** | The very best streams YouTube offers, kept exactly as-is |
| **M4A** | Audio only |

Nothing is re-encoded: YouTube sends video and audio separately, and ffmpeg
just packs them into one file.

**If downloads start failing,** YouTube probably changed something. Click
**Update yt-dlp** at the bottom of the window.

## Build it yourself

You need Node.js, Rust (MSVC toolchain) and the WebView2 runtime (built into Windows 10/11).

```sh
npm install
npm run tauri dev     # run in development
npm run tauri build   # make the installer in src-tauri/target/release/bundle/nsis/
```

See `docs/TECH_REFERENCE.md` for the details. Releases are built, tested and
published by the **Release** GitHub Actions workflow, which is started by hand. The original browser-based
version is kept at the git tag `v1.0.0-local-web`.

## Privacy

Video Vacuum collects nothing: no analytics, no tracking, no accounts, no
server of its own. It only talks to YouTube (through yt-dlp) when you look up
or download a video, and keeps your history on your own computer. Details,
including how to delete that history: [PRIVACY.md](PRIVACY.md).

## Use it responsibly

Only download videos you have the right to save: your own uploads, openly
licensed videos, or content whose owner allows it. Follow YouTube's Terms of
Service and your local copyright law. This project isn't affiliated with
YouTube or Google.

## License

Copyright (C) 2026 TheMysticTestudine

Video Vacuum is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, either version 3 of the License, or (at your option) any later
version. It comes with no warranty. See [LICENSE](LICENSE) for the full text.

Versions up to `v1.0.0-local-web` were released under the MIT License.

The tools Video Vacuum runs (yt-dlp, FFmpeg, Deno) are separate programs with
their own licenses. Video Vacuum doesn't include them.
