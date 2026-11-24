// Shown only when a bundled tool is missing or older than Video Vacuum needs
// (D-32), e.g. if antivirus removed a file. Says what each tool does, links
// its official repository, and offers to restore the exact pinned version
// (downloaded from its official release and checksum-checked).

import { api, type ToolsStatus } from "../api";

interface Props {
  tools: ToolsStatus;
  restoring: boolean;
  message: string;
  onRestore: () => void;
}

export default function ToolsBanner({ tools, restoring, message, onRestore }: Props) {
  const problems = tools.tools.filter((tool) => tool.state !== "ok");
  return (
    <section className="panel notice tools-banner" role="alert">
      <p>
        <strong>Video Vacuum needs {problems.length === 1 ? "one of its tools" : "some of its tools"} put back.</strong>{" "}
        They come with the app, but{" "}
        {problems.length === 1 ? "this one is" : "these are"} missing or older than this version of Video Vacuum needs:
      </p>
      <ul className="tools-list">
        {problems.map((tool) => (
          <li key={tool.id}>
            <strong>{tool.name}</strong>:{" "}
            {tool.state === "missing" ? "missing" : `version ${tool.installed}, needs ${tool.required} or newer`}.{" "}
            <span className="muted">
              {tool.purpose} ({tool.license})
            </span>{" "}
            <button type="button" className="link" onClick={() => api.openProjectPage(`tool:${tool.id}`)}>
              Official repository
            </button>
          </li>
        ))}
      </ul>
      <p className="muted small">
        <strong>Restore</strong> downloads the exact versions this app was built with, from each project's official
        release, and checks them against their known checksums before using them. Or reinstall Video Vacuum, which
        puts them all back too.
      </p>
      <div className="actions">
        <button type="button" onClick={onRestore} disabled={restoring}>
          {restoring ? "Restoring…" : "Restore"}
        </button>
      </div>
      {message && <p className="message error">{message}</p>}
    </section>
  );
}
