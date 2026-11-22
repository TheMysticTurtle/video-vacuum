// Small helpers that turn numbers into the words shown on screen.

/** 19 → "0:19", 3725 → "1:02:05" */
export function formatDuration(seconds: number | null): string {
  if (!seconds || seconds < 0) return "";
  const total = Math.round(seconds);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = String(total % 60).padStart(2, "0");
  return h ? `${h}:${String(m).padStart(2, "0")}:${s}` : `${m}:${s}`;
}

export function formatBytes(bytes: number | null): string {
  if (bytes == null) return "";
  if (bytes >= 1e9) return `${(bytes / 1e9).toFixed(1)} GB`;
  if (bytes >= 1e6) return `${(bytes / 1e6).toFixed(1)} MB`;
  return `${Math.max(1, Math.round(bytes / 1e3))} KB`;
}

export function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`;
}

export function formatViews(count: number | null): string {
  if (count == null) return "";
  return `${new Intl.NumberFormat(undefined, { notation: "compact" }).format(count)} views`;
}

/** "2005-04-23" → "Apr 23, 2005" in the user's own date style. */
export function formatUploadDate(isoDate: string | null): string {
  if (!isoDate) return "";
  const [y, m, d] = isoDate.split("-").map(Number);
  return new Date(y, m - 1, d).toLocaleDateString(undefined, { dateStyle: "medium" });
}

export function formatWhen(secondsSince1970: number): string {
  return new Date(secondsSince1970 * 1000).toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

/** The extension a format usually ends up with (WebM can fall back to .mkv). */
export function extensionFor(format: string): string {
  return format === "m4a" ? ".m4a" : format === "mkv" ? ".mkv" : format === "webm" ? ".webm" : ".mp4";
}
