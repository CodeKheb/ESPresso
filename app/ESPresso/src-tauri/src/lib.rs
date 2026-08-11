mod db;
mod mdns;
mod models;
mod server;
mod ws;

use std::time::Duration;

use std::sync::Arc;

use tauri::{AppHandle, Emitter, Manager, State};

use crate::models::{ConnectionStatus, Contact, Device, DiscoveredDevice, HostInfo, Profile};
use crate::ws::{Command, ConnectionManager, EVT_CONTACTS, EVT_DEVICES, EVT_DISCOVERY};

const POT_PORT: u16 = 8080;

// ── read commands ──────────────────────────────────────────────────────────

#[tauri::command]
fn get_device_id(state: State<'_, Arc<ConnectionManager>>) -> String {
    state.db.get_or_create_device_id()
}

#[tauri::command]
fn get_status(state: State<'_, Arc<ConnectionManager>>) -> ConnectionStatus {
    state.status()
}

#[tauri::command]
fn get_host_info(state: State<'_, Arc<ConnectionManager>>) -> HostInfo {
    state.host_info()
}

#[tauri::command]
fn get_profiles(state: State<'_, Arc<ConnectionManager>>) -> Vec<Profile> {
    state.db.get_profiles()
}

#[tauri::command]
fn get_contacts(state: State<'_, Arc<ConnectionManager>>) -> Vec<Contact> {
    state.db.get_contacts()
}

#[tauri::command]
fn get_devices(state: State<'_, Arc<ConnectionManager>>) -> Vec<Device> {
    state.db.get_devices()
}

// ── write commands ─────────────────────────────────────────────────────────

#[tauri::command]
fn add_contact(
    app: AppHandle,
    state: State<'_, Arc<ConnectionManager>>,
    profile: Profile,
) -> Result<Vec<Contact>, String> {
    state.db.add_contact(&profile).map_err(|e| e.to_string())?;
    let contacts = state.db.get_contacts();
    let _ = app.emit(EVT_CONTACTS, contacts.clone());
    Ok(contacts)
}

#[tauri::command]
fn add_device(
    app: AppHandle,
    state: State<'_, Arc<ConnectionManager>>,
    host: String,
) -> Result<Vec<Device>, String> {
    let host = normalize_host(&host)?;
    state.db.upsert_device("ESPresso pot", &host, POT_PORT, "manual");
    let devices = state.db.get_devices();
    let _ = app.emit(EVT_DEVICES, devices.clone());
    Ok(devices)
}

#[tauri::command]
fn remove_device(
    app: AppHandle,
    state: State<'_, Arc<ConnectionManager>>,
    id: i64,
) -> Result<Vec<Device>, String> {
    state.db.remove_device(id);
    let devices = state.db.get_devices();
    let _ = app.emit(EVT_DEVICES, devices.clone());
    Ok(devices)
}

// ── connection commands ────────────────────────────────────────────────────

/// Manually join a specific pot (host or IP) at the standard pot port.
#[tauri::command]
fn connect_to(
    app: AppHandle,
    state: State<'_, Arc<ConnectionManager>>,
    host: String,
) -> Result<(), String> {
    let host = normalize_host(&host)?;
    state.db.upsert_device("ESPresso pot", &host, POT_PORT, "manual");
    let _ = app.emit(EVT_DEVICES, state.db.get_devices());
    state.send_command(Command::JoinHost(host, POT_PORT));
    Ok(())
}

/// Re-scan the network for pots.
#[tauri::command]
fn retry_connection(state: State<'_, Arc<ConnectionManager>>) {
    state.send_command(Command::Rescan);
}

/// Re-scan the network for pots.
#[tauri::command]
fn connect_auto(state: State<'_, Arc<ConnectionManager>>) {
    state.send_command(Command::Rescan);
}

#[tauri::command]
fn send_profile(
    state: State<'_, Arc<ConnectionManager>>,
    profile: Profile,
) -> Result<(), String> {
    if profile.device_id.trim().is_empty() {
        return Err("missing device_id".into());
    }
    state.send_command(Command::SendProfile(profile));
    Ok(())
}

#[tauri::command]
async fn discover_devices(
    app: AppHandle,
    state: State<'_, Arc<ConnectionManager>>,
) -> Result<Vec<DiscoveredDevice>, String> {
    let info = state.host_info();
    let exclude = vec![info.hostname.to_lowercase(), info.instance.to_lowercase()];
    let mdns = state.mdns.clone();
    let found = tauri::async_runtime::spawn_blocking(move || {
        mdns.browse(Duration::from_secs(2), &exclude)
    })
    .await
    .map_err(|e| e.to_string())?;
    let _ = app.emit(EVT_DISCOVERY, found.clone());
    Ok(found)
}

fn normalize_host(raw: &str) -> Result<String, String> {
    let mut h = raw.trim().to_lowercase();
    for prefix in ["ws://", "http://", "https://"] {
        if let Some(rest) = h.strip_prefix(prefix) {
            h = rest.to_string();
        }
    }
    if let Some(rest) = h.strip_suffix("/ws") {
        h = rest.to_string();
    }
    let h = h.trim_end_matches('/').trim().to_string();
    // Drop an explicit port so a pasted URL doesn't produce `host:8080:8080`.
    let host_only = match h.rsplit_once(':') {
        Some((host, port)) if port.chars().all(|c| c.is_ascii_digit()) => host.to_string(),
        _ => h.clone(),
    };
    let host = host_only.trim().to_string();
    if host.is_empty() {
        return Err("host cannot be empty".into());
    }
    Ok(host)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");
            std::fs::create_dir_all(&app_dir).expect("failed to create app data directory");

            let db = db::Db::open(app_dir.join("profiles.db")).expect("failed to open database");
            let manager = Arc::new(ConnectionManager::new(app.handle().clone(), db));
            app.manage(manager.clone());

            let task = manager.clone();
            tauri::async_runtime::spawn(async move { task.run().await });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_device_id,
            get_status,
            get_host_info,
            get_profiles,
            get_contacts,
            get_devices,
            add_contact,
            add_device,
            remove_device,
            connect_to,
            connect_auto,
            retry_connection,
            send_profile,
            discover_devices,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
