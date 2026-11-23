//! Video Vacuum: the commands the window calls (Tauri IPC), and app startup.
//!
//! The page never runs programs or touches files itself. It asks these
//! commands, which validate everything first: links are rebuilt from a video
//! ID, and files are opened only if this app downloaded them (looked up by
//! history ID).

mod activity;
mod download;
mod link;
mod store;
mod tools;
mod ytdlp;

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::ipc::Channel;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_opener::OpenerExt;

use activity::{Activity, Level, LogLine};
use download::{DownloadEvent, DownloadRequest, JobControl};
use store::{HistoryEntry, Store, Theme};
use tools::{Tools, ToolsStatus};
use ytdlp::{Lookup, VideoInfo};

struct AppState {
    tools: Mutex<Tools>,
    jobs: Mutex<HashMap<String, Arc<JobControl>>>,
    store: Mutex<Store>,
    /// The latest lookup, reused by the download so YouTube isn't asked twice.
    lookup: Mutex<Option<Lookup>>,
    /// The diagnostic log (D-31).
    activity: Mutex<Activity>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    // A panic elsewhere must not lock the user out of their history.
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|err| err.to_string())
}

/// Adds a line to the diagnostic log and shows it in the window's Log panel
/// (event "activity"). Saved to a file only if the user turned that on.
fn note(app: &AppHandle, level: Level, text: impl Into<String>, detail: Option<String>) {
    let line = LogLine::now(level, text, detail);
    if let Some(state) = app.try_state::<AppState>() {
        // The same line twice in a row (e.g. the window re-checking the tools) adds nothing.
        if !lock(&state.activity).push(line.clone()) {
            return;
        }
    }
    let _ = app.emit("activity", line);
}

/// Where saved logs go: `%LOCALAPPDATA%\<app id>\logs` (removed by the
/// uninstaller's "Delete the application data").
fn log_dir(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path().app_local_data_dir().map(|dir| dir.join("logs")).map_err(|err| err.to_string())
}

// ─── Tools ──────────────────────────────────────────────────────────────────

#[tauri::command]
async fn check_tools(app: AppHandle, state: State<'_, AppState>) -> Result<ToolsStatus, String> {
    // The tools were found at startup; this only adds yt-dlp's version (~1 s).
    let tools = lock(&state.tools).clone();
    let status = blocking(move || tools.status()).await?;
    log_tools(&app, &status);
    Ok(status)
}

fn log_tools(app: &AppHandle, status: &ToolsStatus) {
    let version = status.ytdlp_version.as_deref().unwrap_or("not found");
    if status.ready {
        note(app, Level::Info, format!("Tools ready: yt-dlp {version}, FFmpeg and Deno found"), None);
    } else {
        note(
            app,
            Level::Warn,
            format!("Missing tools: {}", status.missing.join(", ")),
            Some("Fix: click \"Install missing tools\" at the top of the window (uses winget).".into()),
        );
    }
}

#[tauri::command]
async fn install_tools(app: AppHandle, state: State<'_, AppState>) -> Result<ToolsStatus, String> {
    note(&app, Level::Info, "Installing the missing tools with winget", None);
    let current = lock(&state.tools).clone();
    let result = blocking(move || {
        current.install_missing()?;
        let tools = Tools::find();
        let status = tools.status();
        Ok::<_, String>((tools, status))
    })
    .await?;
    let (tools, status) = match result {
        Ok(done) => done,
        Err(message) => {
            note(&app, Level::Error, "Installing the tools failed", Some(message.clone()));
            return Err(message);
        }
    };
    *lock(&state.tools) = tools;
    log_tools(&app, &status);
    Ok(status)
}

#[derive(Serialize)]
struct UpdateResult {
    version: Option<String>,
    output: String,
}

#[tauri::command]
async fn update_ytdlp(app: AppHandle, state: State<'_, AppState>) -> Result<UpdateResult, String> {
    note(&app, Level::Info, "Updating yt-dlp with winget", None);
    let tools = lock(&state.tools).clone();
    let result: Result<UpdateResult, String> = blocking(move || {
        let output = tools
            .update_command()?
            .output()
            .map_err(|err| format!("Couldn't start the update: {err}"))?;
        let text = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
        // winget draws progress bars with carriage returns; keep only the readable lines.
        let readable: Vec<&str> = text
            .lines()
            .map(|line| line.rsplit('\r').next().unwrap_or("").trim())
            .filter(|line| line.chars().any(char::is_alphanumeric))
            .collect();
        Ok(UpdateResult { version: tools.ytdlp_version(), output: readable.join("\n") })
    })
    .await?;
    match &result {
        Ok(update) => note(
            &app,
            Level::Info,
            format!("yt-dlp is now {}", update.version.as_deref().unwrap_or("(version unknown)")),
            Some(update.output.clone()),
        ),
        Err(message) => note(&app, Level::Error, "Updating yt-dlp failed", Some(message.clone())),
    }
    result
}

// ─── Video info and downloads ───────────────────────────────────────────────

#[tauri::command]
async fn get_info(url: String, app: AppHandle, state: State<'_, AppState>) -> Result<VideoInfo, String> {
    let Some(clean) = link::clean_youtube_url(&url) else {
        note(&app, Level::Warn, "That doesn't look like a YouTube video link.", Some(url));
        return Err("That doesn't look like a YouTube video link.".into());
    };
    note(&app, Level::Info, format!("Looking up {clean}"), None);
    let tools = lock(&state.tools).clone();
    let (lookup, warnings) = match blocking(move || ytdlp::video_info(&tools, &clean)).await? {
        Ok(found) => found,
        Err(error) => {
            note(&app, Level::Error, format!("Lookup failed: {}", error.message), Some(error.detail));
            return Err(error.message);
        }
    };
    for warning in warnings {
        note(&app, Level::Warn, "yt-dlp warning", Some(warning));
    }
    let info = lookup.info.clone();
    let length = info.duration_seconds.map(|s| format!(", {}:{:02}", s as u64 / 60, s as u64 % 60)).unwrap_or_default();
    note(&app, Level::Info, format!("Found \"{}\" by {}{length}", info.title, info.channel), None);
    *lock(&state.lookup) = Some(lookup);
    Ok(info)
}

#[tauri::command]
fn start_download(
    request: DownloadRequest,
    on_event: Channel<DownloadEvent>,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let job_id = download::new_id();
    let temp_dir = app.path().app_cache_dir().map_err(|err| err.to_string())?;
    let prepared = download::prepare(&request, lock(&state.lookup).as_ref(), &temp_dir, &job_id).inspect_err(|message| {
        note(&app, Level::Warn, format!("Download not started: {message}"), None);
    })?;
    let tools = lock(&state.tools).clone();
    let job = Arc::new(JobControl::default());
    lock(&state.jobs).insert(job_id.clone(), job.clone());

    let id = job_id.clone();
    std::thread::spawn(move || {
        let send = |event: DownloadEvent| {
            let _ = on_event.send(event);
        };
        let log = |level: Level, text: String, detail: Option<String>| note(&app, level, text, detail);
        let finished = download::run(&tools, &request, &prepared, &job, &send, &log);
        let state = app.state::<AppState>();
        lock(&state.jobs).remove(&id);
        if let Some(entry) = finished {
            // Saved before "Done" is sent, so the history drawer already has it.
            if let Err(err) = lock(&state.store).add(entry.clone()) {
                note(&app, Level::Warn, "The download wasn't added to history", Some(err));
            }
            send(DownloadEvent::Done { entry });
        }
    });
    Ok(job_id)
}

#[tauri::command]
fn cancel_download(job_id: String, state: State<'_, AppState>) {
    if let Some(job) = lock(&state.jobs).get(&job_id) {
        job.cancel();
    }
}

// ─── History and settings ───────────────────────────────────────────────────

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HistoryItem {
    #[serde(flatten)]
    entry: HistoryEntry,
    /// False once the file has been moved or deleted.
    exists: bool,
}

#[tauri::command]
async fn list_history(state: State<'_, AppState>) -> Result<Vec<HistoryItem>, String> {
    let entries = lock(&state.store).history.clone();
    blocking(move || {
        entries
            .into_iter()
            .map(|entry| HistoryItem { exists: Path::new(&entry.file_path).is_file(), entry })
            .collect()
    })
    .await
}

/// Removes one entry from history. The downloaded file isn't touched.
#[tauri::command]
fn remove_history_entry(id: String, state: State<'_, AppState>) -> Result<(), String> {
    lock(&state.store).remove(&id)
}

/// Empties history. The downloaded files aren't touched.
#[tauri::command]
fn clear_history(app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    lock(&state.store).clear()?;
    note(&app, Level::Info, "History cleared (downloaded files were not touched)", None);
    Ok(())
}

fn downloaded_file(state: &AppState, id: &str) -> Result<String, String> {
    let path = lock(&state.store).find(id).map(|entry| entry.file_path.clone()).ok_or("Unknown download.")?;
    if Path::new(&path).is_file() {
        Ok(path)
    } else {
        Err("This file has been moved or deleted.".into())
    }
}

#[tauri::command]
fn open_file(id: String, app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let path = downloaded_file(&state, &id)?;
    app.opener().open_path(path, None::<&str>).map_err(|err| err.to_string())
}

#[tauri::command]
fn show_in_folder(id: String, app: AppHandle, state: State<'_, AppState>) -> Result<(), String> {
    let path = downloaded_file(&state, &id)?;
    app.opener().reveal_item_in_dir(path).map_err(|err| err.to_string())
}

/// Open a YouTube page (video or channel) in the default browser. Only
/// https YouTube links are allowed.
#[tauri::command]
fn open_link(url: String, app: AppHandle) -> Result<(), String> {
    let parsed = url::Url::parse(&url).map_err(|_| "Not a link.")?;
    let host = parsed.host_str().unwrap_or("").to_ascii_lowercase();
    let youtube = ["youtube.com", "www.youtube.com", "m.youtube.com", "youtu.be"].contains(&host.as_str());
    if parsed.scheme() != "https" || !youtube {
        return Err("Only YouTube links can be opened.".into());
    }
    app.opener().open_url(parsed.as_str(), None::<&str>).map_err(|err| err.to_string())
}

/// The project's own pages, opened from the bottom bar. The page only names
/// one of these; the addresses are fixed here.
#[tauri::command]
fn open_project_page(page: String, app: AppHandle) -> Result<(), String> {
    const REPO: &str = "https://github.com/TheMysticTurtle/video-vacuum";
    let url = match page.as_str() {
        "github" => REPO.to_string(),
        "privacy" => format!("{REPO}/blob/main/PRIVACY.md"),
        _ => return Err("Unknown page.".into()),
    };
    app.opener().open_url(url, None::<&str>).map_err(|err| err.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsView {
    /// The folder downloads go to unless the user picks another.
    default_folder: String,
    /// True if the user chose it; false = the system Downloads folder.
    is_custom: bool,
    /// False = new downloads aren't added to history.
    keep_history: bool,
    theme: Theme,
    /// True = the diagnostic log is also saved to disk (D-31).
    save_log: bool,
}

fn settings_view(app: &AppHandle, state: &AppState) -> SettingsView {
    let settings = lock(&state.store).settings.clone();
    let chosen = settings.default_folder;
    let system = app
        .path()
        .download_dir()
        .map(|dir| dir.display().to_string())
        .unwrap_or_default();
    SettingsView {
        is_custom: chosen.is_some(),
        default_folder: chosen.unwrap_or(system),
        keep_history: settings.keep_history,
        theme: settings.theme,
        save_log: settings.save_log,
    }
}

#[tauri::command]
fn get_settings(app: AppHandle, state: State<'_, AppState>) -> SettingsView {
    settings_view(&app, &state)
}

/// `None` resets to the system Downloads folder.
#[tauri::command]
fn set_default_folder(folder: Option<String>, app: AppHandle, state: State<'_, AppState>) -> Result<SettingsView, String> {
    if let Some(folder) = &folder {
        if !Path::new(folder).is_dir() {
            return Err("That folder doesn't exist.".into());
        }
    }
    lock(&state.store).set_default_folder(folder)?;
    Ok(settings_view(&app, &state))
}

/// Turning history off keeps the existing list; `clear_history` empties it.
#[tauri::command]
fn set_keep_history(keep: bool, app: AppHandle, state: State<'_, AppState>) -> Result<SettingsView, String> {
    lock(&state.store).set_keep_history(keep)?;
    Ok(settings_view(&app, &state))
}

// ─── Diagnostic log (D-31) ──────────────────────────────────────────────────

/// This session's log so far, for the Log panel when it first opens.
#[tauri::command]
fn get_activity(state: State<'_, AppState>) -> Vec<LogLine> {
    lock(&state.activity).lines()
}

/// Empties the Log panel. Saved log files are kept.
#[tauri::command]
fn clear_activity(state: State<'_, AppState>) {
    lock(&state.activity).clear();
}

/// "Save logs": on = this session's lines and every new line go to a daily
/// file (kept 30 days); off = memory only, gone when the app closes.
#[tauri::command]
fn set_save_log(save: bool, app: AppHandle, state: State<'_, AppState>) -> Result<SettingsView, String> {
    let dir = if save { Some(log_dir(&app)?) } else { None };
    lock(&state.activity).set_saving(dir)?;
    lock(&state.store).set_save_log(save)?;
    note(
        &app,
        Level::Info,
        if save { "Saving logs to disk (kept 30 days)" } else { "Logs are kept for this session only" },
        None,
    );
    Ok(settings_view(&app, &state))
}

#[tauri::command]
fn open_log_folder(app: AppHandle) -> Result<(), String> {
    let dir = log_dir(&app)?;
    std::fs::create_dir_all(&dir).map_err(|err| err.to_string())?;
    app.opener().open_path(dir.display().to_string(), None::<&str>).map_err(|err| err.to_string())
}

/// Deletes every saved log file. The current session's log stays in the panel.
#[tauri::command]
fn delete_saved_logs(app: AppHandle) -> Result<usize, String> {
    let dir = log_dir(&app)?;
    let removed = if dir.is_dir() { activity::delete_saved(&dir)? } else { 0 };
    note(&app, Level::Info, format!("Deleted {removed} saved log file(s)"), None);
    Ok(removed)
}

/// The window's own theme (title bar and frame). The page switches its colors
/// itself from the same setting. Tauri: `Window::set_theme`, None = follow Windows.
fn apply_theme(app: &AppHandle, theme: Theme) {
    let theme = match theme {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
    };
    if let Some(window) = app.get_webview_window("main") {
        if let Err(err) = window.set_theme(theme) {
            eprintln!("couldn't set the window theme: {err}");
        }
    }
}

#[tauri::command]
fn set_theme(theme: Theme, app: AppHandle, state: State<'_, AppState>) -> Result<SettingsView, String> {
    lock(&state.store).set_theme(theme)?;
    apply_theme(&app, theme);
    Ok(settings_view(&app, &state))
}

/// The default window (tauri.conf.json, 1000 x 960) shows a looked-up video
/// and all the save options without scrolling. On a smaller screen, shrink it
/// to the screen's usable area (minus the taskbar) and center it (D-30).
fn fit_window_to_screen(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else { return };
    let (Ok(Some(monitor)), Ok(outer), Ok(inner)) = (window.current_monitor(), window.outer_size(), window.inner_size())
    else {
        return;
    };
    let area = monitor.work_area().size;
    if outer.width <= area.width && outer.height <= area.height {
        return;
    }
    let frame_w = outer.width.saturating_sub(inner.width);
    let frame_h = outer.height.saturating_sub(inner.height);
    let size = tauri::PhysicalSize::new(
        inner.width.min(area.width.saturating_sub(frame_w)),
        inner.height.min(area.height.saturating_sub(frame_h)),
    );
    // The window's minimum size (720 x 560) still applies.
    if window.set_size(size).is_ok() {
        let _ = window.center();
    }
}

// ─── Start ──────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            let store = Store::open(data_dir);
            let theme = store.settings.theme;
            let mut log = Activity::default();
            if store.settings.save_log {
                if let Ok(dir) = log_dir(app.handle()) {
                    let _ = log.set_saving(Some(dir)); // also removes files older than 30 days
                }
            }
            app.manage(AppState {
                // Found before the window opens (a quick folder check), so an
                // early paste never finds the tools "missing".
                tools: Mutex::new(Tools::find()),
                jobs: Mutex::new(HashMap::new()),
                store: Mutex::new(store),
                lookup: Mutex::new(None),
                activity: Mutex::new(log),
            });
            let version = app.package_info().version.to_string();
            note(app.handle(), Level::Info, format!("Video Vacuum {version} started"), None);
            apply_theme(app.handle(), theme);
            fit_window_to_screen(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            check_tools,
            install_tools,
            update_ytdlp,
            get_info,
            start_download,
            cancel_download,
            list_history,
            remove_history_entry,
            clear_history,
            open_file,
            show_in_folder,
            open_link,
            open_project_page,
            get_settings,
            set_default_folder,
            set_keep_history,
            set_theme,
            get_activity,
            clear_activity,
            set_save_log,
            open_log_folder,
            delete_saved_logs,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Video Vacuum");
}
