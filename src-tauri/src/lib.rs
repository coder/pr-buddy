mod auth;
mod github;
mod models;
mod notifications;
mod poller;
mod state;

use tauri::Manager;
use tauri::tray::TrayIconBuilder;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(
            tauri_plugin_stronghold::Builder::new(|password| {
                // Use argon2 to hash the password
                use std::hash::{DefaultHasher, Hasher};
                let mut hasher = DefaultHasher::new();
                hasher.write(password.as_bytes());
                let hash = hasher.finish();
                hash.to_le_bytes().to_vec()
            })
            .build(),
        )
        .manage(state::AppState::new())
        .setup(|app| {
            // Build system tray
            let _tray = TrayIconBuilder::new()
                .tooltip("PR Buddy")
                .on_tray_icon_event(|tray, event| {
                    if let tauri::tray::TrayIconEvent::Click { .. } = event {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            // On macOS, set the activation policy to accessory (no dock icon)
            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            // Start background polling
            poller::start_polling(app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth::start_device_flow_cmd,
            auth::poll_for_token_cmd,
            auth::logout_cmd,
            auth::is_authenticated_cmd,
            github::get_pull_requests_cmd,
            github::get_user_info_cmd,
            github::refresh_prs_cmd,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
