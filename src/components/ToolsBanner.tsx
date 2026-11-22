// Shown only when yt-dlp, ffmpeg or deno is missing, with one button to install them.

import type { ToolsStatus } from "../api";

interface Props {
  tools: ToolsStatus;
  installing: boolean;
  message: string;
  onInstall: () => void;
}

export default function ToolsBanner({ tools, installing, message, onInstall }: Props) {
  return (
    <section className="panel notice" role="alert">
      <p>
        <strong>Video Vacuum needs a few free tools:</strong> {tools.missing.join(", ")}.
      </p>
      {tools.canInstall ? (
        <>
          <p className="muted small">
            They install with winget, Windows' own installer. A window opens so you can follow along. Answer its
            questions there, then come back here.
          </p>
          <div className="actions">
            <button type="button" onClick={onInstall} disabled={installing}>
              {installing ? "Installing…" : "Install missing tools"}
            </button>
          </div>
        </>
      ) : (
        <p className="muted small">
          Install them from github.com/yt-dlp/yt-dlp (yt-dlp brings instructions for ffmpeg and deno), then restart Video Vacuum.
        </p>
      )}
      {message && <p className="message error">{message}</p>}
    </section>
  );
}
