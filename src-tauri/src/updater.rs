use serde::Serialize;
use tauri::AppHandle;
use tauri_plugin_updater::UpdaterExt;

/// Check for an available update. Returns true if an update was found.
pub async fn check_for_update(app: &AppHandle) -> Result<bool, Box<dyn std::error::Error>> {
    let updater = app.updater()?;
    match updater.check().await {
        Ok(Some(update)) => {
            eprintln!("[updater] Update available: v{}", update.version);

            // Show a native notification about the update
            use tauri_plugin_notification::NotificationExt;
            let _ = app
                .notification()
                .builder()
                .title("PR Buddy Update Available")
                .body(format!(
                    "Version {} is available. Use 'Check for Updates' in the tray menu to install.",
                    update.version
                ))
                .show();

            Ok(true)
        }
        Ok(None) => {
            eprintln!("[updater] App is up to date");
            Ok(false)
        }
        Err(e) => {
            eprintln!("[updater] Check failed: {}", e);
            Err(e.into())
        }
    }
}

/// Download and install an available update, then restart the app.
pub async fn download_and_install(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let updater = app.updater()?;
    if let Some(update) = updater.check().await? {
        eprintln!("[updater] Downloading v{} ...", update.version);
        update.download_and_install(|_, _| {}, || {}).await?;
        eprintln!("[updater] Installed, restarting...");
        app.restart();
    }
    Ok(())
}

#[derive(Serialize, Clone)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub update_available: bool,
    pub version: Option<String>,
    pub body: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct DownloadProgress {
    pub chunk_length: usize,
    pub content_length: Option<u64>,
}

#[tauri::command]
pub async fn check_for_update_cmd(app: tauri::AppHandle) -> Result<UpdateCheckResult, String> {
    use tauri_plugin_updater::UpdaterExt;

    let current_version = app.config().version.clone().unwrap_or_default();
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => Ok(UpdateCheckResult {
            current_version,
            update_available: true,
            version: Some(update.version.clone()),
            body: update.body.clone(),
        }),
        Ok(None) => Ok(UpdateCheckResult {
            current_version,
            update_available: false,
            version: None,
            body: None,
        }),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn install_update_cmd(app: tauri::AppHandle) -> Result<(), String> {
    use tauri::Emitter;
    use tauri_plugin_updater::UpdaterExt;

    let updater = app.updater().map_err(|e| e.to_string())?;
    if let Some(update) = updater.check().await.map_err(|e| e.to_string())? {
        let app_for_progress = app.clone();
        update
            .download_and_install(
                move |chunk_len, content_len| {
                    let _ = app_for_progress.emit(
                        "update-download-progress",
                        DownloadProgress {
                            chunk_length: chunk_len,
                            content_length: content_len,
                        },
                    );
                },
                || {},
            )
            .await
            .map_err(|e| e.to_string())?;
        app.restart();
    } else {
        return Err("No update available".to_string());
    }
    Ok(())
}
