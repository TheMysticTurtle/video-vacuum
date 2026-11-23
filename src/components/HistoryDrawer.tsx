// The History drawer on the right: the masthead's "History" button opens it.
// Inside, top to bottom: the title and Close; a search box (with a Clear
// button once something is typed); a box with "Keep download history" and
// "Clear history" side by side (the same controls as in Settings, sharing one
// saved value; D-30); and every past download, each with Remove. Removing or clearing only
// forgets entries; downloaded files are never touched (D-28).
// Files that were moved or deleted collapse and grey out; expanding one says
// where it used to be.

import { useEffect, useMemo, useState } from "react";
import { api, errorText, type HistoryItem, type SettingsView } from "../api";
import { formatBytes, formatWhen } from "../format";
import HistoryControls from "./HistoryControls";

interface Props {
  open: boolean;
  onToggle: () => void;
  refreshKey: number; // bumps after each finished download and after clearing
  settings: SettingsView | null;
  onSettingsChange: (settings: SettingsView) => void;
  onHistoryCleared: () => void;
}

export default function HistoryDrawer({ open, onToggle, refreshKey, settings, onSettingsChange, onHistoryCleared }: Props) {
  const [items, setItems] = useState<HistoryItem[]>([]);
  const [search, setSearch] = useState("");
  const [expanded, setExpanded] = useState<string | null>(null);
  const [message, setMessage] = useState("");

  // Re-check which files still exist whenever the drawer opens, after a
  // download, and when the window comes back into focus.
  useEffect(() => {
    if (!open) return;
    let current = true;
    const load = () =>
      api
        .listHistory()
        .then((list) => current && setItems(list))
        .catch((e) => current && setMessage(errorText(e)));
    load();
    window.addEventListener("focus", load);
    return () => {
      current = false;
      window.removeEventListener("focus", load);
    };
  }, [open, refreshKey]);

  const shown = useMemo(() => {
    const words = search.toLowerCase().split(/\s+/).filter(Boolean);
    if (!words.length) return items;
    return items.filter((item) => {
      const text = `${item.title} ${item.channel} ${item.filePath} ${item.format}`.toLowerCase();
      return words.every((word) => text.includes(word));
    });
  }, [items, search]);

  async function remove(id: string) {
    try {
      await api.removeHistoryEntry(id);
      setItems((list) => list.filter((item) => item.id !== id));
      setMessage("");
    } catch (e) {
      setMessage(errorText(e));
    }
  }

  async function run(action: () => Promise<void>) {
    try {
      await action();
    } catch (e) {
      setMessage(errorText(e)); // e.g. the file was moved since the list loaded
      setItems(await api.listHistory().catch(() => items));
    }
  }

  return (
    <aside id="download-history" className="drawer" hidden={!open} aria-label="Download history">
      {open && (
        <div className="drawer-body">
          <div className="drawer-head">
            <h2>Download history</h2>
            <button type="button" className="secondary" onClick={onToggle}>Close</button>
          </div>
          <div className="with-suffix search-box">
            <input
              type="text"
              placeholder="Search downloads"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              onKeyDown={(e) => e.key === "Escape" && setSearch("")}
              aria-label="Search downloads"
              spellCheck={false}
            />
            {search && (
              <button type="button" className="link search-clear" onClick={() => setSearch("")}>
                Clear
              </button>
            )}
          </div>

          {settings && (
            <HistoryControls
              settings={settings}
              onSettingsChange={onSettingsChange}
              onCleared={onHistoryCleared}
              canClear={items.length > 0}
            />
          )}
          {message && <p className="message error">{message}</p>}

          <ul className="history-list">
            {shown.map((item) =>
              item.exists ? (
                <li key={item.id} className="history-item">
                  {item.thumbnail ? (
                    <img src={item.thumbnail} alt="" referrerPolicy="no-referrer" />
                  ) : (
                    <div className="history-thumb" />
                  )}
                  <div className="history-text">
                    <span className="history-title" title={item.title}>
                      {item.title}
                    </span>
                    <span className="muted small">
                      {[item.format, item.quality, formatBytes(item.sizeBytes)].filter(Boolean).join(" · ")}
                    </span>
                    <span className="muted small">{formatWhen(item.downloadedAt)}</span>
                    <div className="history-actions">
                      <button type="button" className="link" onClick={() => run(() => api.openFile(item.id))}>
                        Open
                      </button>
                      <button type="button" className="link" onClick={() => run(() => api.showInFolder(item.id))}>
                        Show in folder
                      </button>
                      <button type="button" className="link" onClick={() => remove(item.id)}
                        aria-label={`Remove "${item.title}" from history`}>
                        Remove
                      </button>
                    </div>
                  </div>
                </li>
              ) : (
                <li key={item.id} className="history-item missing">
                  <button
                    type="button"
                    className="missing-toggle"
                    onClick={() => setExpanded(expanded === item.id ? null : item.id)}
                    aria-expanded={expanded === item.id}
                  >
                    <span className="history-title">{item.title}</span>
                  </button>
                  {expanded === item.id && (
                    <p className="muted small missing-detail">
                      Moved from {item.filePath}
                      <br />
                      {formatWhen(item.downloadedAt)}
                      <br />
                      <button type="button" className="link" onClick={() => remove(item.id)}
                        aria-label={`Remove "${item.title}" from history`}>
                        Remove
                      </button>
                    </p>
                  )}
                </li>
              ),
            )}
          </ul>
          {!shown.length && <p className="muted small">{items.length ? "No matches." : "Nothing downloaded yet."}</p>}
        </div>
      )}
    </aside>
  );
}
