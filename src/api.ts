// The window's only way to do anything: typed calls to the Rust commands in
// src-tauri/src/lib.rs. Field names match the Rust structs (camelCase).

import { Channel, invoke } from "@tauri-apps/api/core";

export type Format = "mp4" | "webm" | "mkv" | "m4a" | "mp3";

/** MP3 bitrates in kbit/s; must match MP3_BITRATES in src-tauri/src/ytdlp.rs. */
export const MP3_BITRATES = [128, 192, 256, 320] as const;
export const MP3_DEFAULT_KBPS = 192;

export interface ToolsStatus {
  ready: boolean;
  missing: string[];
  ytdlpVersion: string | null;
  canInstall: boolean;
}

export interface VideoInfo {
  url: string;
  id: string;
  title: string;
  channel: string;
  channelUrl: string | null;
  thumbnail: string | null;
  durationSeconds: number | null;
  uploadDate: string | null; // YYYY-MM-DD
  viewCount: number | null;
  description: string;
  heights: number[];
  suggestedFileName: string;
}

export interface HistoryEntry {
  id: string;
  title: string;
  videoUrl: string;
  channel: string;
  thumbnail: string | null;
  format: string;
  quality: string;
  filePath: string;
  sizeBytes: number | null;
  downloadedAt: number; // seconds since 1970
}

export interface HistoryItem extends HistoryEntry {
  exists: boolean; // false once the file was moved or deleted
}

export interface SettingsView {
  defaultFolder: string;
  isCustom: boolean;
  keepHistory: boolean; // false = new downloads aren't added to history
  theme: Theme;
}

/** "system" follows Windows. Saved in settings.json (D-30). */
export type Theme = "system" | "light" | "dark";

// Only the choices: the video itself is the one the app just looked up (Rust keeps it).
export interface DownloadRequest {
  videoId: string;
  format: Format;
  height: number | null; // null = best available
  mp3Kbps: number | null; // MP3 only; null = the default
  folder: string;
  fileName: string; // without extension
}

export type DownloadEvent =
  | { kind: "started"; fileName: string }
  | { kind: "progress"; stage: "video" | "audio"; percent: number; speed: number | null; eta: number | null }
  | { kind: "finishing" }
  | { kind: "done"; entry: HistoryEntry }
  | { kind: "failed"; message: string; detail: string }
  | { kind: "cancelled" };

export const api = {
  checkTools: () => invoke<ToolsStatus>("check_tools"),
  installTools: () => invoke<ToolsStatus>("install_tools"),
  updateYtdlp: () => invoke<{ version: string | null; output: string }>("update_ytdlp"),
  getInfo: (url: string) => invoke<VideoInfo>("get_info", { url }),
  startDownload: (request: DownloadRequest, onEvent: (event: DownloadEvent) => void) => {
    const channel = new Channel<DownloadEvent>();
    channel.onmessage = onEvent;
    return invoke<string>("start_download", { request, onEvent: channel });
  },
  cancelDownload: (jobId: string) => invoke<void>("cancel_download", { jobId }),
  listHistory: () => invoke<HistoryItem[]>("list_history"),
  removeHistoryEntry: (id: string) => invoke<void>("remove_history_entry", { id }),
  clearHistory: () => invoke<void>("clear_history"),
  openFile: (id: string) => invoke<void>("open_file", { id }),
  showInFolder: (id: string) => invoke<void>("show_in_folder", { id }),
  openLink: (url: string) => invoke<void>("open_link", { url }),
  getSettings: () => invoke<SettingsView>("get_settings"),
  setDefaultFolder: (folder: string | null) => invoke<SettingsView>("set_default_folder", { folder }),
  setKeepHistory: (keep: boolean) => invoke<SettingsView>("set_keep_history", { keep }),
  setTheme: (theme: Theme) => invoke<SettingsView>("set_theme", { theme }),
};

/** Commands reject with a plain string message. */
export function errorText(error: unknown): string {
  return typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
}
