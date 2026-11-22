// Step 1: paste a link. Pasting (or Enter, or the Paste button) looks it up.

interface Props {
  value: string;
  onChange: (value: string) => void;
  onSubmit: (value: string) => void;
  busy: boolean;
  message: string;
  isError: boolean;
}

export default function LinkBox({ value, onChange, onSubmit, busy, message, isError }: Props) {
  async function pasteFromClipboard() {
    try {
      const text = await navigator.clipboard.readText();
      onChange(text);
      onSubmit(text);
    } catch {
      document.getElementById("link")?.focus();
    }
  }

  return (
    <section className="panel">
      <label htmlFor="link" className="label">
        Link
      </label>
      <div className="row">
        <input
          id="link"
          type="text"
          inputMode="url"
          autoComplete="off"
          spellCheck={false}
          placeholder="Paste a YouTube link"
          autoFocus
          value={value}
          onChange={(e) => onChange(e.target.value)}
          onPaste={(e) => {
            const text = e.clipboardData.getData("text");
            if (text) {
              e.preventDefault();
              onChange(text);
              onSubmit(text);
            }
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter") onSubmit(value);
          }}
        />
        <button type="button" className="secondary" onClick={pasteFromClipboard} disabled={busy}>
          Paste
        </button>
      </div>
      {message && (
        <p className={isError ? "message error" : "message"} role="status">
          {message}
        </p>
      )}
    </section>
  );
}
