use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

use crate::state::AppState;

const SETTINGS_FILE: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub notify_checks_failed: bool,
    pub notify_checks_passed: bool,
    pub notify_merged: bool,
    pub notify_removed_from_queue: bool,
    /// Repos to hide from the PR panel, as "owner/repo" strings.
    pub hidden_repos: Vec<String>,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            notify_checks_failed: true,
            notify_checks_passed: true,
            notify_merged: true,
            notify_removed_from_queue: true,
            hidden_repos: vec![],
        }
    }
}

/// Load from disk, falling back to defaults.
pub fn load_settings(app_data_dir: &PathBuf) -> UserSettings {
    let path = app_data_dir.join(SETTINGS_FILE);
    match fs::read_to_string(&path) {
        Ok(contents) => serde_json::from_str(&contents).unwrap_or_default(),
        Err(_) => UserSettings::default(),
    }
}

/// Save to disk.
pub fn save_settings(app_data_dir: &PathBuf, settings: &UserSettings) -> Result<(), String> {
    if let Err(e) = fs::create_dir_all(app_data_dir) {
        return Err(format!("Failed to create app data dir: {}", e));
    }
    let path = app_data_dir.join(SETTINGS_FILE);
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))
}

#[tauri::command]
pub async fn get_settings_cmd(state: State<'_, AppState>) -> Result<UserSettings, String> {
    let settings = state.settings.lock().unwrap().clone();
    Ok(settings)
}

#[tauri::command]
pub async fn save_settings_cmd(
    app: AppHandle,
    state: State<'_, AppState>,
    settings: UserSettings,
) -> Result<(), String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("Failed to resolve app data dir: {}", e))?;
    save_settings(&app_data_dir, &settings)?;
    *state.settings.lock().unwrap() = settings;
    Ok(())
}
