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
