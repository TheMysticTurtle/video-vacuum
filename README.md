# Video Saver

A tiny, no-fuss app for saving YouTube videos. Paste a link, pick a format and
quality, click **Download**.

It runs **on your own computer**. A small local server drives
[yt-dlp](https://github.com/yt-dlp/yt-dlp) for you and shows a simple page in
your browser. Nothing is uploaded anywhere, and nothing else on your network
can reach it.

## Quick start

### Windows

1. Download this repo (**Code → Download ZIP**) and unzip it.
2. Double-click **`Start.bat`**.
   - If something is missing, it offers to install it with `winget`. Say **Y**,
     then double-click `Start.bat` again when it finishes.
3. Your browser opens the app. Keep the black window open while you use it.
   Close that window to quit.

### macOS / Linux

```sh
./start.sh
```

If something is missing, it prints the exact install command
(macOS: `brew install python yt-dlp ffmpeg deno`).

## Using it

1. **Paste a YouTube link** into the box (or click **Paste**). Messy links are
   fine. Tracking bits like `?si=…`, `&t=…` and `&list=…` are trimmed off, and
   Shorts, `youtu.be` and mobile links all work.
2. Check the video: thumbnail, title, creator (click to open their channel),
   and the clean link with a **Copy link** button.
3. Pick a **Format** and **Quality**, then click **Download**. Files go to
   your **Downloads** folder. Click **Show in folder** when it's done.

| Format | What you get |
|---|---|
| **MP4** | Most compatible. H.264 video + AAC audio up to 1080p (above 1080p YouTube only offers VP9/AV1). |
| **WebM** | VP9/AV1 video + Opus audio. |
| **MKV** | The very best streams YouTube offers, kept exactly as-is. |
| **M4A** | Audio only. |

Nothing is re-encoded. YouTube sends video and audio separately, and ffmpeg
just packs them into one file, so there's no quality loss.

**Quality** lists only the resolutions that video actually has. "Best
available" picks the highest one.

## Keeping it working

YouTube changes things often, which breaks downloads until yt-dlp catches up.
If downloads start failing, click **Update yt-dlp** at the bottom of the page.

## What it needs

| Tool | Why | Installed by |
|---|---|---|
| Python 3.9+ | runs the app (standard library only, no `pip install`) | `Start.bat` / your package manager |
| yt-dlp | does the downloading | `winget install yt-dlp.yt-dlp` |
| ffmpeg | joins video + audio | comes with the yt-dlp winget package |
| deno | yt-dlp needs a JavaScript runtime for full YouTube support | comes with the yt-dlp winget package |

## Use it responsibly

Only download videos you have the right to save, such as your own uploads,
openly licensed videos, or content whose owner allows it. Follow YouTube's
Terms of Service and your local copyright law. This project isn't affiliated
with YouTube or Google.

## License

[MIT](LICENSE)
