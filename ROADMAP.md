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
| `[?]` | **Owner's hands-on walkthrough** of the installed app | waiting on the owner |
| `[x]` | **Pushed** `main` + tag `v1.0.0-local-web` (2026-11-22) | done |
| `[x]` | **Release workflow** `.github/workflows/release.yml` (owner 2026-11-22: "builds it. tests it. and makes a release when i run it manually") | done |
| `[?]` | **First run** of the Release workflow → GitHub release v2.0.0 | waiting on the owner |

## 2. Next

Nothing queued. The owner picks from the backlog.

## 3. Milestones

- `[x]` v1.0.0 local web edition (2026-10-25, tag `v1.0.0-local-web`)
- `[x]` v2.0.0 desktop app: paste, card, format/quality/name/folder, progress + cancel, History drawer with default folder and search, update button, tools check, installer
- `[ ]` Public release (the owner decides when the repo goes public)

## 4. Polish checklist

- `[ ]` Code signing, so Windows' "Windows protected your PC" warning goes away (costs money every year; the owner's call)

## 5. Decisions waiting on the owner

- **Banner look:** transparent (logo on navy, current) or keep the glow background (`tools/make-brand-images.ps1 -BannerGlow`)?
- **Merge `feature/dark-studio`** (GPT's restyle + light/dark + brand images) into `main`?
- **Upload `docs/images/social-preview.jpg`** in GitHub → Settings → Social preview (GitHub has no API for it).
- Optional: a simplified 16 px icon (the detailed logo blurs at title-bar size).

- Make the GitHub repo public?
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
