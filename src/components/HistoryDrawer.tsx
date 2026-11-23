// The History drawer on the right: the masthead's "History" button opens it.
// Inside: Settings (default folder, keep history on/off, clear history), a
// search box, and every past download, each with Remove. Removing or clearing
// only forgets the entries; the downloaded files are never touched (D-28).
// Files that were moved or deleted collapse and grey out; expanding one says
// where it used to be.

import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useEffect, useMemo, useState } from "react";
import { api, errorText, type HistoryItem, type SettingsView } from "../api";
import { formatBytes, formatWhen } from "../format";

interface Props {
  open: boolean;
  onToggle: () => void;
  refreshKey: number; // bumps after each finished download
  settings: SettingsView | null;
  onSettingsChange: (settings: SettingsView) => void;
}

export default function HistoryDrawer({ open, onToggle, refreshKey, settings, onSettingsChange }: Props) {
  const [items, setItems] = useState<HistoryItem[]>([]);
  const [search, setSearch] = useState("");
  const [showSettings, setShowSettings] = useState(false);
  const [confirmClear, setConfirmClear] = useState(false);
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

  async function changeFolder() {
    const folder = await openDialog({ directory: true, defaultPath: settings?.defaultFolder, title: "Default download folder" });
    if (typeof folder !== "string") return;
    try {
      onSettingsChange(await api.setDefaultFolder(folder));
      setMessage("");
    } catch (e) {
      setMessage(errorText(e));
    }
  }

  async function resetFolder() {
    try {
      onSettingsChange(await api.setDefaultFolder(null));
    } catch (e) {
      setMessage(errorText(e));
    }
  }

  async function setKeepHistory(keep: boolean) {
    try {
      onSettingsChange(await api.setKeepHistory(keep));
      setMessage("");
    } catch (e) {
      setMessage(errorText(e));
    }
  }

  async function clearHistory() {
    setConfirmClear(false);
    try {
      await api.clearHistory();
      setItems([]);
      setExpanded(null);
      setMessage("");
    } catch (e) {
      setMessage(errorText(e));
    }
  }

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
          <button type="button" className="link settings-toggle" onClick={() => setShowSettings(!showSettings)}
            aria-expanded={showSettings}>Settings</button>

          {showSettings && settings && (
            <div className="settings">
              <span className="label">Default download folder</span>
              <p className="folder-path" title={settings.defaultFolder}>
                {settings.defaultFolder}
              </p>
              <div className="row">
                <button type="button" className="secondary" onClick={changeFolder}>
                  Change…
                </button>
                {settings.isCustom && (
                  <button type="button" className="secondary" onClick={resetFolder}>
                    Use Downloads
                  </button>
                )}
              </div>

              <label className="check">
                <input
                  type="checkbox"
                  checked={settings.keepHistory}
                  onChange={(e) => setKeepHistory(e.target.checked)}
                />
                Keep download history
              </label>

              {confirmClear ? (
                <div className="confirm" role="alert">
                  <p className="small">Clear all history? This can't be undone. Your downloaded files stay where they are.</p>
                  <div className="row">
                    <button type="button" className="danger" onClick={clearHistory}>
                      Clear
                    </button>
                    <button type="button" className="secondary" onClick={() => setConfirmClear(false)}>
                      Cancel
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  type="button"
                  className="secondary"
                  onClick={() => setConfirmClear(true)}
                  disabled={!items.length}
                >
                  Clear history…
                </button>
              )}
            </div>
          )}

          {settings && !settings.keepHistory && (
            <p className="muted small">History is off. New downloads aren't recorded.</p>
          )}

          <input
            type="search"
            placeholder="Search downloads"
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            aria-label="Search downloads"
          />
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
