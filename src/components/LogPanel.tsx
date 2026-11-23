// The diagnostic log (D-31): a panel docked above the bottom bar, opened with
// the bar's "Log" toggle. The rest of the window stays usable, so you can
// watch it while you work. Drag its top edge to resize it (remembered).
//
// Lines come from Rust (event "activity"); this panel only shows them. It
// collects lines even while closed, so opening it shows the whole session.

import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { api, errorText, type LogLine } from "../api";

const KEEP_LINES = 2000; // matches activity.rs
const MIN_HEIGHT = 120;
const HEIGHT_KEY = "videoVacuum.logHeight"; // a per-window convenience, not a setting

interface Props {
  open: boolean;
  onClose: () => void;
  saving: boolean; // "Save logs" is on
}

function savedHeight(): number {
  try {
    const value = Number(localStorage.getItem(HEIGHT_KEY));
    return value >= MIN_HEIGHT ? value : 220;
  } catch {
    return 220;
  }
}

const time = (atMs: number) => new Date(atMs).toLocaleTimeString(undefined, { hour12: false });

function asText(line: LogLine): string {
  const detail = line.detail ? `\n    ${line.detail.replace(/\n/g, "\n    ")}` : "";
  return `${time(line.atMs)}  ${line.level.toUpperCase().padEnd(5)}  ${line.text}${detail}`;
}

export default function LogPanel({ open, onClose, saving }: Props) {
  const [lines, setLines] = useState<LogLine[]>([]);
  const [height, setHeight] = useState(savedHeight);
  const [message, setMessage] = useState("");
  const listRef = useRef<HTMLDivElement>(null);
  const stickToBottom = useRef(true);

  // Load this session's lines, then follow new ones.
  useEffect(() => {
    let active = true;
    const unlisten = listen<LogLine>("activity", (event) => {
      if (active) setLines((current) => [...current, event.payload].slice(-KEEP_LINES));
    });
    api
      .getActivity()
      .then((earlier) => active && setLines((current) => [...earlier, ...current.filter((l) => l.atMs > (earlier.at(-1)?.atMs ?? 0))]))
      .catch(() => undefined);
    return () => {
      active = false;
      unlisten.then((stop) => stop());
    };
  }, []);

  // Keep the newest line in view, unless the user scrolled up to read.
  useEffect(() => {
    const list = listRef.current;
    if (open && list && stickToBottom.current) list.scrollTop = list.scrollHeight;
  }, [lines, open, height]);

  function onScroll() {
    const list = listRef.current;
    if (list) stickToBottom.current = list.scrollHeight - list.scrollTop - list.clientHeight < 24;
  }

  function startResize(event: React.PointerEvent<HTMLDivElement>) {
    event.preventDefault();
    const handle = event.currentTarget;
    try {
      handle.setPointerCapture(event.pointerId); // keep the drag even if the pointer leaves the handle
    } catch {
      // no capture (e.g. the pointer is already gone): the drag still works over the handle
    }
    const startY = event.clientY;
    const startHeight = height;
    const max = () => Math.max(MIN_HEIGHT, Math.round(window.innerHeight * 0.7));
    let latest = startHeight;
    const move = (e: PointerEvent) => {
      latest = Math.min(max(), Math.max(MIN_HEIGHT, startHeight + (startY - e.clientY)));
      setHeight(latest);
    };
    const end = () => {
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", end);
      handle.removeEventListener("pointercancel", end);
      try {
        localStorage.setItem(HEIGHT_KEY, String(latest));
      } catch {
        // storage unavailable: the size just isn't remembered
      }
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", end);
    handle.addEventListener("pointercancel", end);
  }

  function resizeWithKeys(event: React.KeyboardEvent<HTMLDivElement>) {
    const step = event.key === "ArrowUp" ? 24 : event.key === "ArrowDown" ? -24 : 0;
    if (!step) return;
    event.preventDefault();
    setHeight((h) => Math.min(Math.round(window.innerHeight * 0.7), Math.max(MIN_HEIGHT, h + step)));
  }

  async function copy() {
    try {
      await navigator.clipboard.writeText(lines.map(asText).join("\n"));
      setMessage("Copied.");
    } catch (e) {
      setMessage(errorText(e));
    }
  }

  async function clear() {
    await api.clearActivity().catch(() => undefined);
    setLines([]);
    setMessage("");
  }

  return (
    <section id="log-panel" className="log-panel" hidden={!open} style={{ height }} aria-label="Log">
      <div
        className="log-resize"
        role="separator"
        aria-orientation="horizontal"
        aria-label="Resize the log"
        tabIndex={0}
        onPointerDown={startResize}
        onKeyDown={resizeWithKeys}
      />
      <div className="log-head">
        <strong>Log</strong>
        <span className="muted small">{saving ? "Saved to disk (30 days)" : "This session only"}</span>
        {message && <span className="muted small">{message}</span>}
        <span className="log-actions">
          <button type="button" className="secondary small-button" onClick={copy} disabled={!lines.length}>
            Copy
          </button>
          <button type="button" className="secondary small-button" onClick={clear} disabled={!lines.length}>
            Clear
          </button>
          <button type="button" className="secondary small-button" onClick={onClose}>
            Close
          </button>
        </span>
      </div>
      <div className="log-lines" ref={listRef} onScroll={onScroll} role="log" aria-live="polite">
        {lines.map((line, i) => (
          <div key={`${line.atMs}-${i}`} className={`log-line ${line.level}`}>
            <span className="log-time">{time(line.atMs)}</span>
            <span className="log-text">
              {line.text}
              {line.detail && <pre className="log-detail">{line.detail}</pre>}
            </span>
          </div>
        ))}
        {!lines.length && <p className="muted small">Nothing yet. Steps appear here as the app works.</p>}
      </div>
    </section>
  );
}
