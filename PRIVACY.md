# Privacy

**Video Vacuum doesn't collect any personal information.** It has no
analytics, no tracking, no ads, no accounts, and no server of its own. The
developer never receives anything from it.

Everything below happens on your own computer, or directly between your
computer and the services named, only when you use that feature.

_Last updated: 2026-11-23 (version 2.1.0)._

## What leaves your computer

| When you… | Your computer talks to | What they can see |
|---|---|---|
| Look up or download a video | **YouTube** (through the open-source [yt-dlp](https://github.com/yt-dlp/yt-dlp) tool) | Your IP address and which video, the same as watching it in a web browser. Video Vacuum signs in to nothing and sends no cookies. |
| See a video's thumbnail | YouTube's image server (`i.ytimg.com`), the only outside address the app's window is allowed to load | The same as above |
| Click **Update yt-dlp** | GitHub, through yt-dlp's own updater, which downloads the newest yt-dlp from its official releases | A normal software download |
| Click **Restore** (only shown if a bundled tool is missing) | GitHub, to download that tool's exact pinned version from its official release | A normal software download |
| Click a video title or channel name | Your web browser opens the page on YouTube | Whatever your browser and YouTube normally see |

**Parts of Windows that Video Vacuum relies on:**
- The app's window is drawn by **Microsoft Edge WebView2**, which is built
  into Windows. Microsoft describes what WebView2 sends in
  [Data and privacy in WebView2](https://learn.microsoft.com/microsoft-edge/webview2/concepts/data-privacy):
  some required reliability data and crash reports, and optional diagnostics
  that follow your setting in **Windows Settings → Privacy & security →
  Diagnostics & feedback**. Video Vacuum doesn't add to or read this data.
- **Windows SmartScreen** may check the installer when you run it, as it does
  for any downloaded program.

## What stays on your computer

| What | Where | Why |
|---|---|---|
| Your downloaded files | The folder you choose (your Downloads folder unless you change it) | They're what you asked for |
| Download history: titles, video and channel links, thumbnail links, file locations, sizes, dates | `%APPDATA%\com.themysticturtle.videovacuum\history.json` | The **History** drawer and its search |
| Your settings: default download folder, theme, and whether to keep history | `%APPDATA%\com.themysticturtle.videovacuum\settings.json` | So you don't have to pick them every time |
| The window's own cache, which may include thumbnails | `%LOCALAPPDATA%\com.themysticturtle.videovacuum\` | Kept by WebView2 |
| A temporary copy of a video's details | The same local folder | Lets the download reuse the lookup; deleted as soon as the download finishes |
| The diagnostic **Log**: what the app did, with video titles, links, file locations and yt-dlp's messages | In memory only, unless you turn on **Settings → Save logs to disk**. Then: `%LOCALAPPDATA%\com.themysticturtle.videovacuum\logs\`, one file per day, deleted after 30 days | So you can see what happened and why something failed |

None of this is sent anywhere.

## Deleting your data

- **In the app:** open **History** or **Settings**.
  - **Clear history** deletes the whole list (it asks first).
  - **Remove** on any entry deletes just that one.
  - Untick **Keep download history** and new downloads aren't recorded at all.
  - None of these touch your downloaded files.
  - **Settings → Delete saved logs** removes every saved log file. With
    **Save logs to disk** off (the default), the log disappears when you close the app.
- **When uninstalling:** tick **"Delete the application data"** in the
  uninstaller. That removes both `com.themysticturtle.videovacuum` folders
  above. Without the tick, your history is kept in case you reinstall.
- **Any time:** close Video Vacuum, then delete those two folders. Paste
  `%APPDATA%` or `%LOCALAPPDATA%` into File Explorer's address bar to find them.
- Your downloaded videos are ordinary files; delete them like any other file.

## Questions

Video Vacuum is open source, so you can read exactly what it does in this
repository. Questions or concerns: open an issue at
https://github.com/TheMysticTurtle/video-vacuum/issues.
