// The Settings panel on the right, opened by the masthead's "Settings" button
// (D-30): default download folder, theme (System / Light / Dark) and the same
// history controls the History panel shows. Every value lives in settings.json
// and comes back from the Rust command that saved it.

import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { useState } from "react";
import { api, errorText, type SettingsView, type Theme } from "../api";
import HistoryControls from "./HistoryControls";

interface Props {
  open: boolean;
  onClose: () => void;
  settings: SettingsView | null;
  onSettingsChange: (settings: SettingsView) => void;
  onHistoryCleared: () => void;
}

const THEMES: { value: Theme; label: string }[] = [
  { value: "system", label: "System" },
  { value: "light", label: "Light" },
  { value: "dark", label: "Dark" },
];

export default function SettingsDrawer({ open, onClose, settings, onSettingsChange, onHistoryCleared }: Props) {
  const [message, setMessage] = useState("");

  async function save(change: () => Promise<SettingsView>) {
    try {
      onSettingsChange(await change());
      setMessage("");
    } catch (e) {
      setMessage(errorText(e));
    }
  }

  async function changeFolder() {
    const folder = await openDialog({ directory: true, defaultPath: settings?.defaultFolder, title: "Default download folder" });
    if (typeof folder === "string") await save(() => api.setDefaultFolder(folder));
  }

  return (
    <aside id="settings-panel" className="drawer" hidden={!open} aria-label="Settings">
      {open && settings && (
        <div className="drawer-body">
          <div className="drawer-head">
            <h2>Settings</h2>
            <button type="button" className="secondary" onClick={onClose}>Close</button>
          </div>

          <section className="settings">
            <span className="label">Default download folder</span>
            <p className="folder-path" title={settings.defaultFolder}>
              {settings.defaultFolder}
            </p>
            <div className="row">
              <button type="button" className="secondary" onClick={changeFolder}>
                Change…
              </button>
              {settings.isCustom && (
                <button type="button" className="secondary" onClick={() => save(() => api.setDefaultFolder(null))}>
                  Use Downloads
                </button>
              )}
            </div>
          </section>

          <section className="settings">
            <span className="label" id="theme-label">Theme</span>
            <div className="segmented" role="radiogroup" aria-labelledby="theme-label">
              {THEMES.map((theme) => (
                <label key={theme.value} className={settings.theme === theme.value ? "selected" : ""}>
                  <input
                    type="radio"
                    name="theme"
                    value={theme.value}
                    checked={settings.theme === theme.value}
                    onChange={() => save(() => api.setTheme(theme.value))}
                  />
                  {theme.label}
                </label>
              ))}
            </div>
            <p className="muted small">System follows your Windows setting.</p>
          </section>

          <section className="settings">
            <span className="label">History</span>
            <HistoryControls settings={settings} onSettingsChange={onSettingsChange} onCleared={onHistoryCleared} />
          </section>

          {message && <p className="message error">{message}</p>}
        </div>
      )}
    </aside>
  );
}
