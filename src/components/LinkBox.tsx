interface Props {
  value: string;
  onChange: (value: string) => void;
  onSubmit: (value: string) => void;
  busy: boolean;
  locked: boolean;
  compact: boolean;
  message: string;
  isError: boolean;
}

export default function LinkBox({ value, onChange, onSubmit, busy, locked, compact, message, isError }: Props) {
  async function pasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      if (text.trim()) {
        onChange(text);
        onSubmit(text);
      } else {
        document.getElementById("link")?.focus();
      }
    } catch {
      document.getElementById("link")?.focus();
    }
  }

  return (
    <section className={compact ? "source compact" : "source"}>
      {!compact && (
        <div className="hero">
          <div className="hero-copy">
            <h1>New download</h1>
            <p>Save a YouTube video or just its audio.</p>
          </div>
          <div className="reel-art" aria-hidden="true">
            <div className="film-tail" />
            <div className="reel" />
          </div>
        </div>
      )}
      <form className="source-form" onSubmit={(e) => { e.preventDefault(); if (!busy && !locked) onSubmit(value); }}>
        <label htmlFor="link" className="label">Video link</label>
        <div className="row">
          <input id="link" type="text" inputMode="url" autoComplete="off" spellCheck={false}
            placeholder="Paste a YouTube link" autoFocus value={value} disabled={locked}
            onChange={(e) => onChange(e.target.value)}
            onPaste={(e) => {
              const text = e.clipboardData.getData("text");
              if (text.trim()) {
                e.preventDefault();
                onChange(text);
                onSubmit(text);
              }
            }}
          />
          {value.trim() && !compact ? (
            <button type="submit" disabled={busy || locked}>{busy ? "Looking up…" : "Look up"}</button>
          ) : (
            <button type="button" onClick={pasteFromClipboard} disabled={busy || locked}>Paste link</button>
          )}
        </div>
        {message && <p className={isError ? "message error" : "message"} role="status">{message}</p>}
      </form>
    </section>
  );
}
