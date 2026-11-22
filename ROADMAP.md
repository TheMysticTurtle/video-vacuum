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
| `[?]` | **SmartScreen "incorrect warning" report** for the installer link. Filled in 2026-11-22 except name, email, street address and the CAPTCHA, which only the owner can do; then Submit | waiting on the owner |

## 2. Next

Nothing queued. The owner picks from the backlog.

## 3. Milestones

- `[x]` v1.0.0 local web edition (2026-10-25, tag `v1.0.0-local-web`)
- `[x]` v2.0.0 desktop app: paste, card, format/quality/name/folder, progress + cancel, History drawer with default folder and search, update button, tools check, installer
- `[x]` Public release: v2.0.0 on GitHub, repo public (2026-11-22)

## 4. Polish checklist

- `[ ]` Code signing, so Windows' "Windows protected your PC" warning goes away (costs money every year; the owner's call). Without it, each new version starts with no SmartScreen reputation, and Windows 11 Smart App Control blocks it outright for people who have that turned on.
- `[ ]` Optional: submit the installer itself to Microsoft Security Intelligence as a "Software developer" (the file-reputation route; the SmartScreen form above is the website route).

## 5. Decisions waiting on the owner

- **Upload `docs/images/social-preview.jpg`** in GitHub → Settings → Social preview (GitHub has no API for it). Still the default image as of 2026-11-22.
- **GitHub repo description** still says "A tiny local web UI for yt-dlp", which describes v1. Suggested: "Paste a YouTube link, pick format and quality, download. A small Windows app for yt-dlp; runs on your computer, nothing uploaded."
- Optional: a simplified 16 px icon (the detailed logo blurs at title-bar size).
- LICENSE holder name: git name "TheMysticTestudine", GitHub "TheMysticTurtle", or a real name.

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
