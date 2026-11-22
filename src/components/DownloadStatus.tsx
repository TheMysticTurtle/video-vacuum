// Progress while downloading, then the result: open it, or try again.

import { useState } from "react";
import { api, type HistoryEntry } from "../api";
import { formatSpeed } from "../format";

export type DownloadState =
  | { phase: "idle" }
  | { phase: "starting"; jobId: string | null }
  | {
      phase: "downloading";
      jobId: string;
      fileName: string;
      stage: "video" | "audio";
      percent: number;
      speed: number | null;
      eta: number | null;
    }
  | { phase: "finishing"; jobId: string; fileName: string }
  | { phase: "done"; entry: HistoryEntry }
  | { phase: "failed"; message: string; detail: string }
  | { phase: "cancelled" };

interface Props {
  state: DownloadState;
  onCancel: () => void;
  onAnother: () => void;
}

function timeLeft(seconds: number): string {
  const m = Math.floor(seconds / 60);
  return `${m}:${String(seconds % 60).padStart(2, "0")} left`;
}

export default function DownloadStatus({ state, onCancel, onAnother }: Props) {
  const [showDetail, setShowDetail] = useState(false);
  if (state.phase === "idle") return null;

  const running = state.phase === "starting" || state.phase === "downloading" || state.phase === "finishing";
  const percent = state.phase === "downloading" ? state.percent : state.phase === "finishing" || state.phase === "done" ? 100 : 0;

  let text = "Starting…";
  if (state.phase === "downloading") {
    const parts = [`Downloading ${state.stage}… ${state.percent.toFixed(0)}%`];
    if (state.speed) parts.push(formatSpeed(state.speed));
    if (state.eta != null) parts.push(timeLeft(state.eta));
    text = parts.join(" · ");
  } else if (state.phase === "finishing") {
    text = "Finishing up…";
  } else if (state.phase === "done") {
    text = `Saved as ${state.entry.filePath.split(/[\\/]/).pop()}`;
  } else if (state.phase === "failed") {
    text = state.message;
  } else if (state.phase === "cancelled") {
    text = "Cancelled. Nothing was kept.";
  }

  return (
    <section className="panel" aria-live="polite">
      {(running || state.phase === "done") && <progress max={100} value={percent} />}
      <p className={state.phase === "failed" ? "message error" : "message"}>{text}</p>
      {state.phase === "failed" && state.detail && (
        <>
          <button type="button" className="link" onClick={() => setShowDetail(!showDetail)}>
            {showDetail ? "Hide details" : "Details"}
          </button>
          {showDetail && <pre className="detail">{state.detail}</pre>}
        </>
      )}
      <div className="actions">
        {running && (
          <button type="button" className="secondary" onClick={onCancel}>
            Cancel
          </button>
        )}
        {state.phase === "done" && (
          <>
            <button type="button" className="secondary" onClick={() => api.openFile(state.entry.id)}>
              Open
            </button>
            <button type="button" className="secondary" onClick={() => api.showInFolder(state.entry.id)}>
              Show in folder
            </button>
            <button type="button" onClick={onAnother}>
              Download another
            </button>
          </>
        )}
      </div>
    </section>
  );
}
