// Video Saver — page logic.
// Talks to the small local server in app.py; all the real work is done there by yt-dlp.

"use strict";

const $ = (id) => document.getElementById(id);

let currentVideo = null;   // info for the video on screen (from /api/info)
let infoRequest = 0;       // ignores slow answers for a link that was since replaced
let currentJob = null;     // id of the download in progress / just finished

// ─── Talking to the server ──────────────────────────────────────────────────

// GET when there's no body, POST (JSON) when there is. Throws with the server's message on failure.
async function api(path, body) {
  const options = body === undefined ? {} : {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  };
  const response = await fetch(path, options);
  const data = await response.json().catch(() => ({}));
  if (!response.ok) throw new Error(data.error || `Request failed (${response.status}).`);
  return data;
}

function showMessage(element, text, isError = false) {
  element.textContent = text;
  element.classList.toggle("error", isError);
}

// Only ever link to real https:// pages (never "javascript:" etc.).
function safeLink(url) {
  return /^https:\/\//i.test(url || "") ? url : null;
}

// ─── Step 1 + 2: read the link, show the video ──────────────────────────────

async function lookUpLink() {
  const text = $("link").value.trim();
  if (!text) return;
  const request = ++infoRequest;

  $("video").hidden = $("options").hidden = true;
  showMessage($("link-message"), "Looking up video…");

  try {
    const video = await api("/api/info", { url: text });
    if (request !== infoRequest) return;          // a newer link was pasted meanwhile
    currentVideo = video;
    showVideo(video);
    showMessage($("link-message"), "");
  } catch (error) {
    if (request !== infoRequest) return;
    showMessage($("link-message"), error.message, true);
  }
}

function showVideo(video) {
  $("link").value = video.url;                    // swap in the cleaned-up link
  $("clean-link").value = video.url;

  $("thumbnail").src = safeLink(video.thumbnail) || "";
  $("title").textContent = video.title;
  $("title").href = video.url;
  $("channel").textContent = video.channel;
  const channelUrl = safeLink(video.channel_url);
  if (channelUrl) $("channel").href = channelUrl; else $("channel").removeAttribute("href");
  $("duration").textContent = video.duration ? `Length: ${video.duration}` : "";

  // Quality menu: "Best available" plus every resolution this video really has.
  const quality = $("quality");
  quality.replaceChildren(new Option("Best available", ""));
  for (const height of video.heights) quality.add(new Option(`${height}p`, String(height)));

  $("progress-area").hidden = true;
  $("download").disabled = false;
  $("video").hidden = $("options").hidden = false;
  updateQualityMenu();
}

// Audio-only downloads don't have a resolution.
function updateQualityMenu() {
  $("quality").disabled = $("format").value === "m4a";
}

// ─── Step 3: download + progress ────────────────────────────────────────────

async function startDownload() {
  if (!currentVideo) return;
  const height = $("quality").value ? Number($("quality").value) : null;

  $("download").disabled = true;
  $("show-file").hidden = true;
  $("progress-area").hidden = false;
  $("progress").value = 0;
  showMessage($("progress-text"), "Starting…");

  try {
    const { job } = await api("/api/download", {
      url: currentVideo.url, format: $("format").value, height,
    });
    currentJob = job;
    watchJob(job);
  } catch (error) {
    showMessage($("progress-text"), error.message, true);
    $("download").disabled = false;
  }
}

// Ask the server for progress twice a second until the download ends.
async function watchJob(job) {
  try {
    const state = await api(`/api/job?id=${encodeURIComponent(job)}`);
    showProgress(state);
    if (state.status === "done" || state.status === "error") {
      $("download").disabled = false;
      return;
    }
  } catch (error) {
    showMessage($("progress-text"), error.message, true);
    $("download").disabled = false;
    return;
  }
  setTimeout(() => watchJob(job), 500);
}

function showProgress(state) {
  const text = $("progress-text");
  $("progress").value = state.percent;

  if (state.status === "downloading") {
    const parts = [`Downloading ${state.stage}… ${state.percent.toFixed(0)}%`];
    if (state.speed) parts.push(formatSpeed(state.speed));
    if (state.eta != null) parts.push(`${formatTime(state.eta)} left`);
    showMessage(text, parts.join(" · "));
  } else if (state.status === "finishing") {
    showMessage(text, "Finishing up…");
  } else if (state.status === "done") {
    showMessage(text, `Saved: ${state.file.split(/[\\/]/).pop()}`);
    $("show-file").hidden = false;
  } else if (state.status === "error") {
    showMessage(text, state.error, true);
  } else {
    showMessage(text, "Starting…");
  }
}

function formatSpeed(bytesPerSecond) {
  const mb = bytesPerSecond / 1_000_000;
  return mb >= 1 ? `${mb.toFixed(1)} MB/s` : `${(bytesPerSecond / 1000).toFixed(0)} KB/s`;
}

function formatTime(seconds) {
  const m = Math.floor(seconds / 60);
  const s = String(Math.floor(seconds % 60)).padStart(2, "0");
  return `${m}:${s}`;
}

// ─── Footer: yt-dlp version + update ────────────────────────────────────────

async function loadVersion() {
  try {
    $("version").textContent = (await api("/api/version")).version;
  } catch {
    $("version").textContent = "unknown";
  }
}

async function updateYtDlp() {
  $("update").disabled = true;
  showMessage($("update-message"), "Updating yt-dlp… (this can take a minute)");
  try {
    const result = await api("/api/update", {});
    $("version").textContent = result.version;
    showMessage($("update-message"), result.output || "Done.");
  } catch (error) {
    showMessage($("update-message"), error.message, true);
  } finally {
    $("update").disabled = false;
  }
}

// ─── Wiring up the buttons ──────────────────────────────────────────────────

// Pasting into the box looks the video up straight away (after the text lands).
$("link").addEventListener("paste", () => setTimeout(lookUpLink, 0));
$("link").addEventListener("keydown", (event) => {
  if (event.key === "Enter") lookUpLink();
});

// "Paste" button: read the clipboard (the browser may ask for permission once).
$("paste").addEventListener("click", async () => {
  try {
    $("link").value = await navigator.clipboard.readText();
    lookUpLink();
  } catch {
    $("link").focus();
    showMessage($("link-message"), "Couldn't read the clipboard — click the box and press Ctrl+V (⌘V on Mac).");
  }
});

$("copy").addEventListener("click", async () => {
  try {
    await navigator.clipboard.writeText($("clean-link").value);
    $("copy").textContent = "Copied!";
  } catch {
    $("clean-link").select();                     // fallback: select it for Ctrl+C
    $("copy").textContent = "Press Ctrl+C";
  }
  setTimeout(() => { $("copy").textContent = "Copy link"; }, 1500);
});

$("format").addEventListener("change", updateQualityMenu);
$("download").addEventListener("click", startDownload);
$("show-file").addEventListener("click", () => api("/api/open", { job: currentJob }).catch(() => {}));
$("update").addEventListener("click", updateYtDlp);

loadVersion();
