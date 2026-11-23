// "Keep download history" and "Clear history…", shown in two places: above the
// list in the History panel and in the Settings panel (D-30). Both copies read
// the same saved setting (settings.json → keepHistory, via the app's one
// `settings` state) and change it through the same command, so they always
// agree. Clearing only forgets the list; downloaded files are never touched (D-28).

import { useState } from "react";
import { api, errorText, type SettingsView } from "../api";

interface Props {
  settings: SettingsView;
  onSettingsChange: (settings: SettingsView) => void;
  /** Called after the list was cleared, so any open list reloads. */
  onCleared: () => void;
  /** False greys out "Clear history…" (the History panel knows when the list is empty). */
  canClear?: boolean;
}

export default function HistoryControls({ settings, onSettingsChange, onCleared, canClear = true }: Props) {
  const [confirmClear, setConfirmClear] = useState(false);
  const [message, setMessage] = useState("");

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
      setMessage("");
      onCleared();
    } catch (e) {
      setMessage(errorText(e));
    }
  }

  return (
    <div className="history-controls">
      <label className="check">
        <input type="checkbox" checked={settings.keepHistory} onChange={(e) => setKeepHistory(e.target.checked)} />
        Keep download history
      </label>
      {!settings.keepHistory && <p className="muted small">History is off. New downloads aren't recorded.</p>}

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
        <button type="button" className="secondary" onClick={() => setConfirmClear(true)} disabled={!canClear}>
          Clear history…
        </button>
      )}
      {message && <p className="message error">{message}</p>}
    </div>
  );
}
