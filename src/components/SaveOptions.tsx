// Step 3: format, quality, file name and folder, then Download.

import { open } from "@tauri-apps/plugin-dialog";
import { MP3_BITRATES, type Format, type VideoInfo } from "../api";
import { extensionFor } from "../format";

export interface SaveChoices {
  format: Format;
  height: number | null; // null = best available
  mp3Kbps: number; // used only for MP3
  fileName: string;
  folder: string;
}

interface Props {
  video: VideoInfo;
  choices: SaveChoices;
  onChange: (choices: SaveChoices) => void;
  onDownload: () => void;
  disabled: boolean;
}

export default function SaveOptions({ video, choices, onChange, onDownload, disabled }: Props) {
  const mp3 = choices.format === "mp3";
  const audioOnly = choices.format === "m4a" || mp3;
  const set = (patch: Partial<SaveChoices>) => onChange({ ...choices, ...patch });

  async function chooseFolder() {
    const folder = await open({ directory: true, defaultPath: choices.folder || undefined, title: "Save downloads to" });
    if (typeof folder === "string") set({ folder });
  }

  return (
    <section className="panel">
      <span className="label">Save as</span>
      <div className="grid">
        <label className="field">
          Format
          <select value={choices.format} onChange={(e) => set({ format: e.target.value as Format })} disabled={disabled}>
            <option value="mp4">MP4 · video (most compatible)</option>
            <option value="webm">WebM · video</option>
            <option value="mkv">MKV · video (keeps original streams)</option>
            <option value="m4a">M4A · audio only (original)</option>
            <option value="mp3">MP3 · audio only (converted)</option>
          </select>
        </label>
        {mp3 ? (
          <label className="field">
            Bitrate
            <select
              value={String(choices.mp3Kbps)}
              onChange={(e) => set({ mp3Kbps: Number(e.target.value) })}
              disabled={disabled}
            >
              {MP3_BITRATES.map((kbps) => (
                <option key={kbps} value={kbps}>
                  {kbps === 192 ? `${kbps} kbps (recommended)` : `${kbps} kbps`}
                </option>
              ))}
            </select>
          </label>
        ) : (
          <label className="field">
            Quality
            <select
              value={audioOnly || choices.height == null ? "" : String(choices.height)}
              onChange={(e) => set({ height: e.target.value ? Number(e.target.value) : null })}
              disabled={disabled || audioOnly}
            >
              <option value="">{audioOnly ? "Best audio" : "Best available"}</option>
              {!audioOnly && video.heights.map((h) => <option key={h} value={h}>{`${h}p`}</option>)}
            </select>
          </label>
        )}
        <label className="field wide">
          File name
          <div className="with-suffix">
            <input
              type="text"
              value={choices.fileName}
              onChange={(e) => set({ fileName: e.target.value })}
              spellCheck={false}
              disabled={disabled}
            />
            <span className="suffix">{extensionFor(choices.format)}</span>
          </div>
        </label>
        <div className="field wide">
          Folder
          <div className="row">
            <input type="text" value={choices.folder} readOnly title={choices.folder} aria-label="Folder" />
            <button type="button" className="secondary" onClick={chooseFolder} disabled={disabled}>
              Browse
            </button>
          </div>
        </div>
      </div>
      <div className="actions">
        <button type="button" onClick={onDownload} disabled={disabled || !choices.fileName.trim() || !choices.folder}>
          Download
        </button>
      </div>
    </section>
  );
}
