// Video Vacuum's window: Link → Video → Save as → progress, plus the History drawer.

import { useEffect, useRef, useState } from "react";
import { api, errorText, type DownloadEvent, type SettingsView, type ToolsStatus, type VideoInfo } from "./api";
import DownloadStatus, { type DownloadState } from "./components/DownloadStatus";
import HistoryDrawer from "./components/HistoryDrawer";
// The skinny banner, made from media/ by tools/make-brand-images.ps1.
import headerBanner from "./assets/header-banner.png";
import LinkBox from "./components/LinkBox";
import SaveOptions, { type SaveChoices } from "./components/SaveOptions";
import ToolsBanner from "./components/ToolsBanner";
import VideoCard from "./components/VideoCard";

export default function App() {
  const [tools, setTools] = useState<ToolsStatus | null>(null);
  const [installing, setInstalling] = useState(false);
  const [toolsMessage, setToolsMessage] = useState("");
  const [settings, setSettings] = useState<SettingsView | null>(null);

  const [link, setLink] = useState("");
  const [lookup, setLookup] = useState<{ busy: boolean; message: string; isError: boolean }>({
    busy: false,
    message: "",
    isError: false,
  });
  const [video, setVideo] = useState<VideoInfo | null>(null);
  const lookupId = useRef(0); // ignores slow answers for a link that was since replaced

  const [choices, setChoices] = useState<SaveChoices>({ format: "mp4", height: null, fileName: "", folder: "" });
  const [download, setDownload] = useState<DownloadState>({ phase: "idle" });
  const [historyOpen, setHistoryOpen] = useState(false);
  const [historyKey, setHistoryKey] = useState(0);
  const [update, setUpdate] = useState({ busy: false, message: "" });

  useEffect(() => {
    api.checkTools().then(setTools).catch((e) => setToolsMessage(errorText(e)));
    api.getSettings().then(setSettings).catch(() => undefined);
  }, []);

  const running = download.phase === "starting" || download.phase === "downloading" || download.phase === "finishing";

  async function lookUp(text: string) {
    if (!text.trim() || running) return;
    const id = ++lookupId.current;
    setVideo(null);
    setDownload({ phase: "idle" });
    setLookup({ busy: true, message: "Looking up video…", isError: false });
    try {
      const info = await api.getInfo(text);
      if (id !== lookupId.current) return;
      setLink(info.url); // show the cleaned-up link
      setVideo(info);
      setChoices((c) => ({ ...c, fileName: info.suggestedFileName, folder: settings?.defaultFolder ?? c.folder }));
      setLookup({ busy: false, message: "", isError: false });
    } catch (e) {
      if (id !== lookupId.current) return;
      setLookup({ busy: false, message: errorText(e), isError: true });
    }
  }

  function onEvent(event: DownloadEvent) {
    if (event.kind === "done") setHistoryKey((k) => k + 1); // the drawer re-reads its list
    setDownload((current) => {
      const jobId = "jobId" in current && current.jobId ? current.jobId : "";
      const fileName = "fileName" in current ? current.fileName : "";
      switch (event.kind) {
        case "started":
          return { phase: "starting", jobId: jobId || null }; // "Starting…" until real progress arrives
        case "progress":
          return { phase: "downloading", jobId, fileName, stage: event.stage, percent: event.percent, speed: event.speed, eta: event.eta };
        case "finishing":
          return { phase: "finishing", jobId, fileName };
        case "done":
          return { phase: "done", entry: event.entry };
        case "failed":
          return { phase: "failed", message: event.message, detail: event.detail };
        case "cancelled":
          return { phase: "cancelled" };
      }
    });
  }

  async function startDownload() {
    if (!video || running) return;
    setDownload({ phase: "starting", jobId: null });
    try {
      const jobId = await api.startDownload(
        {
          videoId: video.id,
          format: choices.format,
          height: choices.format === "m4a" ? null : choices.height,
          folder: choices.folder,
          fileName: choices.fileName,
        },
        onEvent,
      );
      // Events may arrive before the job ID does; attach it to whatever state we're in.
      setDownload((current) => ("jobId" in current ? ({ ...current, jobId } as DownloadState) : current));
    } catch (e) {
      setDownload({ phase: "failed", message: errorText(e), detail: "" });
    }
  }

  function cancel() {
    if ("jobId" in download && download.jobId) api.cancelDownload(download.jobId);
  }

  function another() {
    lookupId.current++;
    setLink("");
    setVideo(null);
    setDownload({ phase: "idle" });
    document.getElementById("link")?.focus();
  }

  async function install() {
    setInstalling(true);
    setToolsMessage("");
    try {
      const status = await api.installTools();
      setTools(status);
      if (!status.ready) setToolsMessage(`Still missing: ${status.missing.join(", ")}.`);
    } catch (e) {
      setToolsMessage(errorText(e));
    } finally {
      setInstalling(false);
    }
  }

  async function updateYtdlp() {
    setUpdate({ busy: true, message: "Updating yt-dlp… (this can take a minute)" });
    try {
      const before = tools?.ytdlpVersion;
      const result = await api.updateYtdlp();
      setTools((t) => (t ? { ...t, ytdlpVersion: result.version } : t));
      // Plain words for the common outcomes; the full output otherwise.
      const message =
        result.version && result.version !== before
          ? `Updated to ${result.version}.`
          : /no available upgrade|up to date|up-to-date/i.test(result.output)
            ? "yt-dlp is already up to date."
            : result.output || "Done.";
      setUpdate({ busy: false, message });
    } catch (e) {
      setUpdate({ busy: false, message: errorText(e) });
    }
  }

  function changeSettings(next: SettingsView) {
    // Follow the new default unless a download folder was picked for this video.
    setChoices((c) => (c.folder === settings?.defaultFolder || !c.folder ? { ...c, folder: next.defaultFolder } : c));
    setSettings(next);
  }

  return (
    <div className={historyOpen ? "layout drawer-open" : "layout"}>
      <header className="masthead">
        <div className="brand"><img src={headerBanner} alt="Video Vacuum" /></div>
        <button type="button" className="secondary history-toggle" aria-expanded={historyOpen}
          aria-controls="download-history" onClick={() => setHistoryOpen(!historyOpen)}>
          <span aria-hidden="true">◷</span> History
        </button>
      </header>
      <main>
        {!tools && toolsMessage && <p className="message error" role="alert">{toolsMessage}</p>}
        {tools && !tools.ready && (
          <ToolsBanner tools={tools} installing={installing} message={toolsMessage} onInstall={install} />
        )}

        <LinkBox
          compact={!!video}
          value={link}
          onChange={(text) => {
            lookupId.current++;
            setLink(text);
            setVideo(null);
            setDownload({ phase: "idle" });
            setLookup({ busy: false, message: "", isError: false });
          }}
          onSubmit={lookUp}
          busy={lookup.busy}
          locked={running}
          message={lookup.message}
          isError={lookup.isError}
        />

        {video && <VideoCard video={video} />}
        {video && (
          <SaveOptions
            video={video}
            choices={choices}
            onChange={setChoices}
            onDownload={startDownload}
            disabled={running || !tools?.ready}
          />
        )}
        <DownloadStatus state={download} onCancel={cancel} onAnother={another} />

        <footer>
          <div className="engine">
            <span className={tools?.ready ? "indicator ready" : "indicator"} />
            <strong>{tools?.ready ? "Engine ready" : tools ? "Setup required" : toolsMessage ? "Engine unavailable" : "Checking tools…"}</strong>
          </div>
          <span>
            yt-dlp {tools ? (tools.ytdlpVersion ?? "not found") : "…"}
            {tools?.ytdlpVersion && (
              <>
                {" · "}
                <button type="button" className="link" onClick={updateYtdlp} disabled={update.busy || running}>
                  Update yt-dlp
                </button>
              </>
            )}
          </span>
          {update.message && <p className="message pre">{update.message}</p>}
          {/* GPLv3 "Appropriate Legal Notices" (§0, §5d) */}
          <p className="small legal">
            Video Vacuum © 2026 TheMysticTestudine. Free software under the GNU GPL v3 or later, with no warranty. Full
            license: gnu.org/licenses/gpl-3.0
          </p>
        </footer>
      </main>

      <HistoryDrawer
        open={historyOpen}
        onToggle={() => setHistoryOpen(!historyOpen)}
        refreshKey={historyKey}
        settings={settings}
        onSettingsChange={changeSettings}
      />
    </div>
  );
}
