//! Video Vacuum: the commands the window calls (Tauri IPC), and app startup.
//!
//! The page never runs programs or touches files itself. It asks these
//! commands, which validate everything first: links are rebuilt from a video
//! ID, and files are opened only if this app downloaded them (looked up by
//! history ID).

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
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;

use download::{DownloadEvent, DownloadRequest, JobControl};
use store::{HistoryEntry, Store};
use tools::{Tools, ToolsStatus};
use ytdlp::{Lookup, VideoInfo};

struct AppState {
    tools: Mutex<Tools>,
    jobs: Mutex<HashMap<String, Arc<JobControl>>>,
    store: Mutex<Store>,
    /// The latest lookup, reused by the download so YouTube isn't asked twice.
    lookup: Mutex<Option<Lookup>>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    // A panic elsewhere must not lock the user out of their history.
    mutex.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

async fn blocking<T: Send + 'static>(work: impl FnOnce() -> T + Send + 'static) -> Result<T, String> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|err| err.to_string())
}

// ─── Tools ──────────────────────────────────────────────────────────────────

#[tauri::command]
async fn check_tools(state: State<'_, AppState>) -> Result<ToolsStatus, String> {
    // The tools were found at startup; this only adds yt-dlp's version (~1 s).
    let tools = lock(&state.tools).clone();
    blocking(move || tools.status()).await
}

#[tauri::command]
async fn install_tools(state: State<'_, AppState>) -> Result<ToolsStatus, String> {
    let current = lock(&state.tools).clone();
    let (tools, status) = blocking(move || {
        current.install_missing()?;
        let tools = Tools::find();
        let status = tools.status();
        Ok::<_, String>((tools, status))
    })
    .await??;
    *lock(&state.tools) = tools;
    Ok(status)
}

#[derive(Serialize)]
struct UpdateResult {
    version: Option<String>,
    output: String,
}

#[tauri::command]
async fn update_ytdlp(state: State<'_, AppState>) -> Result<UpdateResult, String> {
    let tools = lock(&state.tools).clone();
    blocking(move || {
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
    .await?
}

// ─── Video info and downloads ───────────────────────────────────────────────

#[tauri::command]
async fn get_info(url: String, state: State<'_, AppState>) -> Result<VideoInfo, String> {
    let clean = link::clean_youtube_url(&url).ok_or("That doesn't look like a YouTube video link.")?;
    let tools = lock(&state.tools).clone();
    let lookup = blocking(move || ytdlp::video_info(&tools, &clean)).await??;
    let info = lookup.info.clone();
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
    let prepared = download::prepare(&request, lock(&state.lookup).as_ref(), &temp_dir, &job_id)?;
    let tools = lock(&state.tools).clone();
    let job = Arc::new(JobControl::default());
    lock(&state.jobs).insert(job_id.clone(), job.clone());

    let id = job_id.clone();
    std::thread::spawn(move || {
        let send = |event: DownloadEvent| {
            let _ = on_event.send(event);
        };
        let finished = download::run(&tools, &request, &prepared, &job, &send);
        let state = app.state::<AppState>();
        lock(&state.jobs).remove(&id);
        if let Some(entry) = finished {
            // Saved before "Done" is sent, so the history drawer already has it.
            if let Err(err) = lock(&state.store).add(entry.clone()) {
                eprintln!("history not saved: {err}");
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SettingsView {
    /// The folder downloads go to unless the user picks another.
    default_folder: String,
    /// True if the user chose it; false = the system Downloads folder.
    is_custom: bool,
}

fn settings_view(app: &AppHandle, state: &AppState) -> SettingsView {
    let chosen = lock(&state.store).settings.default_folder.clone();
    let system = app
        .path()
        .download_dir()
        .map(|dir| dir.display().to_string())
        .unwrap_or_default();
    SettingsView { is_custom: chosen.is_some(), default_folder: chosen.unwrap_or(system) }
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

// ─── Start ──────────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            app.manage(AppState {
                // Found before the window opens (a quick folder check), so an
                // early paste never finds the tools "missing".
                tools: Mutex::new(Tools::find()),
                jobs: Mutex::new(HashMap::new()),
                store: Mutex::new(Store::open(data_dir)),
                lookup: Mutex::new(None),
            });
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
            open_file,
            show_in_folder,
            open_link,
            get_settings,
            set_default_folder,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Video Vacuum");
}
