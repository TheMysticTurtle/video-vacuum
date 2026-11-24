# Changelog

What changed in each version of Video Vacuum, newest first. Downloads are on
the [Releases](https://github.com/TheMysticTurtle/video-vacuum/releases) page.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

## [2.2.0] - 2026-11-24

- **Everything in one installer:** yt-dlp, Deno and a small FFmpeg are built
  in, so there's no separate setup and no winget windows. About 56 MB,
  instead of roughly 600 MB of separately installed tools.
- **A tool check:** if a tool is ever missing or out of date, a banner
  explains it and offers **Restore**. That downloads the exact official
  version and checks its checksum.
- **Update yt-dlp** now uses yt-dlp's own updater.
- **Fixed:** an MP3 download could delete an existing WebM (or similar file)
  with the same name. Downloads now pick a new name instead, like
  "Video (2).mp3".

## [2.1.0] - 2026-11-23

- **MP3:** save audio as MP3 at 128–320 kbps (192 recommended).
- **History privacy:** clear all history (it asks first), turn history off,
  or remove single entries. Your downloaded files are never touched.
- **Settings panel:** default folder, **theme** (System / Light / Dark),
  history and log options.
- **Log:** open it from the bottom bar to watch each step, with yt-dlp's own
  words when something fails. It's kept for the session only, or saved for
  30 days if you choose.
- **A bottom status bar** with **Privacy Policy** and **GitHub** links.
- **Layout polish:** bigger section titles, a description box with an arrow
  toggle, and a window sized to show everything.

> ⚠️ In 2.1, downloading an MP3 could delete an existing WebM file with the
> same name. Fixed in 2.2; please update.

## [2.0.0] - 2026-11-22

Version 1.0 was a small local web page you started with `Start.bat`. 2.0
rebuilds it as a real Windows app:

- **A proper installer and Start Menu app.** No Python, browser tab or
  `Start.bat` anymore.
- **Paste any YouTube link.** Messy share links, Shorts, `youtu.be` and
  mobile links are cleaned up automatically.
- **A video card:** thumbnail, title, creator, upload date, length, views and
  description.
- **Pick format (MP4, WebM, MKV, M4A), quality, file name and folder.**
  Progress with Cancel, and it never overwrites a file.
- **History panel** with search and a default download folder setting. Files
  you've moved or deleted are greyed out.
- **One-click setup** of the free tools it needs (yt-dlp, FFmpeg, Deno)
  through winget, plus an **Update yt-dlp** button.
- **Faster downloads:** about twice as fast on short videos in testing,
  because it reuses the lookup instead of asking YouTube twice.
- **A new name and look:** *Video Vacuum* (it was *Video Saver*), with its own
  art. It follows Windows' light/dark setting.
- **Now free software under GPL-3.0-or-later** (1.0 was MIT).

## [1.0.0] - 2026-10-25 (local web edition, never released)

Kept as the git tag `v1.0.0-local-web`, under the MIT License.

- A small Python server on your own computer (127.0.0.1 only) with a page in
  your browser, started with `Start.bat` (Windows) or `start.sh`.
- Paste a YouTube link: messy links trimmed to the plain watch link. See the
  creator, thumbnail and a link to the video.
- MP4, WebM, MKV or M4A at the quality you pick, plus **Update yt-dlp**.

[2.2.0]: https://github.com/TheMysticTurtle/video-vacuum/releases/tag/v2.2.0
[2.1.0]: https://github.com/TheMysticTurtle/video-vacuum/releases/tag/v2.1.0
[2.0.0]: https://github.com/TheMysticTurtle/video-vacuum/releases/tag/v2.0.0
[1.0.0]: https://github.com/TheMysticTurtle/video-vacuum/tree/v1.0.0-local-web
