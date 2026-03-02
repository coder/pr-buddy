mod auth;
mod github;
mod menu;
mod models;
mod notifications;
mod poller;
mod settings;
mod state;
mod updater;

use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(
            tauri_plugin_stronghold::Builder::new(|password| {
                use sha2::{Sha256, Digest};
                let hash = Sha256::digest(password.as_bytes());
                hash.to_vec()
            })
            .build(),
        )
        .manage(state::AppState::new())
        .invoke_handler(tauri::generate_handler![
            auth::start_device_flow_cmd,
            auth::poll_for_token_cmd,
            auth::logout_cmd,
            auth::is_authenticated_cmd,
            github::get_pull_requests_cmd,
            github::get_user_info_cmd,
            github::refresh_prs_cmd,
            settings::get_settings_cmd,
            settings::save_settings_cmd
        ])
        .setup(|app| {
            // Restore saved auth token from disk (if any)
            if let Some(saved_token) = auth::load_token_from_disk(app.handle()) {
                let state = app.state::<state::AppState>();
                *state.token.lock().unwrap() = Some(saved_token);
                eprintln!("[setup] Restored auth session from disk");
            }

            // Load user settings from disk
            if let Ok(app_data_dir) = app.path().app_data_dir() {
                let loaded = settings::load_settings(&app_data_dir);
                let state = app.state::<state::AppState>();
                *state.settings.lock().unwrap() = loaded;
                eprintln!("[setup] Loaded user settings from disk");
            }

            // Build initial menu based on auth state
            let state = app.state::<state::AppState>();
            let is_authed = state.token.lock().unwrap().is_some();
            let initial_menu = if is_authed {
                let prs = state.prs.lock().unwrap();
                menu::build_pr_menu(app.handle(), &prs)?
            } else {
                menu::build_auth_menu(app.handle())?
            };

            // Build system tray — left-click opens native menu
            let tray_icon = tauri::image::Image::from_bytes(
                include_bytes!("../icons/tray-default.png"),
            )
            .expect("failed to load tray icon");

            let tray = TrayIconBuilder::new()
                .icon(tray_icon)
                .icon_as_template(true)
                .tooltip("PR Buddy")
                .menu(&initial_menu)
                .show_menu_on_left_click(true)
                .on_menu_event(|app, event| {
                    handle_menu_event(app, event.id.as_ref());
                })
                .build(app)?;

            // Store tray handle so poller can update the menu
            *state.tray.lock().unwrap() = Some(tray);

            // Request notification permission (triggers macOS permission dialog)
            {
                use tauri_plugin_notification::NotificationExt;
                match app.notification().request_permission() {
                    Ok(state) => eprintln!("[notifications] Permission state: {:?}", state),
                    Err(e) => eprintln!("[notifications] Failed to request permission: {}", e),
                }
            }

            #[cfg(target_os = "macos")]
            {
                app.set_activation_policy(tauri::ActivationPolicy::Accessory);
            }

            poller::start_polling(app.handle().clone());

            // Validate restored token asynchronously (don't block startup).
            // If the token is revoked/invalid, clear it and fall back to sign-in.
            if is_authed {
                let app_handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let token = {
                        let state = app_handle.state::<state::AppState>();
                        let val = state.token.lock().unwrap().clone();
                        val
                    };
                    if let Some(token) = token {
                        match github::validate_token(&token).await {
                            Some(false) => {
                                // Token is confirmed invalid (401/403) — clear it,
                                // but only if the token hasn't been replaced by a
                                // fresh login while we were validating.
                                let state = app_handle.state::<state::AppState>();
                                let mut current = state.token.lock().unwrap();
                                if current.as_deref() == Some(&token) {
                                    eprintln!("[setup] Saved token is invalid, clearing session");
                                    *current = None;
                                    drop(current);
                                    auth::delete_token_from_disk(&app_handle);
                                    {
                                        let tray_guard = state.tray.lock().unwrap();
                                        if let Some(tray) = tray_guard.as_ref() {
                                            if let Ok(m) = menu::build_auth_menu(&app_handle) {
                                                let _ = tray.set_menu(Some(m));
                                            }
                                        }
                                    }
                                    // Notify the frontend so it switches to the login screen
                                    let _ = app_handle.emit("auth-cleared", ());
                                } else {
                                    eprintln!("[setup] Token changed during validation, keeping new session");
                                }
                            }
                            Some(true) => {
                                eprintln!("[setup] Saved token validated successfully");
                            }
                            None => {
                                // Network error — keep the token, poller will retry
                                eprintln!("[setup] Could not validate token (offline?), keeping session");
                            }
                        }
                    }
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn handle_menu_event(app: &tauri::AppHandle, id: &str) {
    match id {
        "quit" => app.exit(0),

        "refresh" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let token = {
                    let state = app.state::<state::AppState>();
                    let val = state.token.lock().unwrap().clone();
                    val
                };
                if let Some(token) = token {
                    if let Ok(prs) = github::fetch_pull_requests(&token).await {
                        let state = app.state::<state::AppState>();
                        *state.prs.lock().unwrap() = prs.clone();
                        let tray_guard = state.tray.lock().unwrap();
                        if let Some(tray) = tray_guard.as_ref() {
                            if let Ok(new_menu) = menu::build_pr_menu(&app, &prs) {
                                let _ = tray.set_menu(Some(new_menu));
                            }
                        }
                    }
                }
            });
        }

        "sign_in" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                match auth::start_device_flow().await {
                    Ok(resp) => {
                        // Copy user_code to clipboard
                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                            let _ = clipboard.set_text(&resp.user_code);
                        }

                        // Open verification URL in browser
                        {
                            use tauri_plugin_opener::OpenerExt;
                            let _ = app.opener().open_url(&resp.verification_uri, None::<&str>);
                        }

                        // Update menu to pending state
                        {
                            let state = app.state::<state::AppState>();
                            let tray_guard = state.tray.lock().unwrap();
                            if let Some(tray) = tray_guard.as_ref() {
                                if let Ok(pending_menu) =
                                    menu::build_auth_pending_menu(&app, &resp.user_code)
                                {
                                    let _ = tray.set_menu(Some(pending_menu));
                                }
                            }
                        }

                        // Poll for token until success or expiry
                        let expires_at = std::time::Instant::now()
                            + std::time::Duration::from_secs(resp.expires_in);
                        let interval = std::time::Duration::from_secs(resp.interval.max(5));

                        loop {
                            tokio::time::sleep(interval).await;
                            if std::time::Instant::now() >= expires_at {
                                eprintln!("[auth] Device flow expired");
                                let state = app.state::<state::AppState>();
                                let tray_guard = state.tray.lock().unwrap();
                                if let Some(tray) = tray_guard.as_ref() {
                                    if let Ok(m) = menu::build_auth_menu(&app) {
                                        let _ = tray.set_menu(Some(m));
                                    }
                                }
                                break;
                            }

                            let state = app.state::<state::AppState>();
                            match auth::poll_for_token(&resp.device_code, &state, &app).await {
                                Ok(true) => {
                                    eprintln!("[auth] ✅ Authenticated via menu");
                                    let token = {
                                        let val = state.token.lock().unwrap().clone();
                                        val
                                    };
                                    if let Some(token) = token {
                                        let prs = github::fetch_pull_requests(&token)
                                            .await
                                            .unwrap_or_default();
                                        *state.prs.lock().unwrap() = prs.clone();
                                        let tray_guard = state.tray.lock().unwrap();
                                        if let Some(tray) = tray_guard.as_ref() {
                                            if let Ok(m) = menu::build_pr_menu(&app, &prs) {
                                                let _ = tray.set_menu(Some(m));
                                            }
                                        }
                                    }
                                    break;
                                }
                                Ok(false) => continue,
                                Err(e) => {
                                    eprintln!("[auth] Token poll error: {}", e);
                                    let tray_guard = state.tray.lock().unwrap();
                                    if let Some(tray) = tray_guard.as_ref() {
                                        if let Ok(m) = menu::build_auth_menu(&app) {
                                            let _ = tray.set_menu(Some(m));
                                        }
                                    }
                                    break;
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[auth] Failed to start device flow: {}", e);
                    }
                }
            });
        }

        "check_updates" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let _ = updater::download_and_install(&app).await;
            });
        }

        id if id.starts_with("pr_") => {
            let pr_id = &id[3..];
            let state = app.state::<state::AppState>();
            let prs = state.prs.lock().unwrap();
            if let Some(pr) = prs.iter().find(|p| p.id == pr_id) {
                use tauri_plugin_opener::OpenerExt;
                let _ = app.opener().open_url(&pr.url, None::<&str>);
            }
        }

        "see_all" => {
            use tauri_plugin_opener::OpenerExt;
            let _ = app
                .opener()
                .open_url("https://github.com/pulls", None::<&str>);
        }

        "logout" => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app.state::<state::AppState>();
                *state.token.lock().unwrap() = None;
                *state.prs.lock().unwrap() = vec![];
                state.previous_prs.lock().unwrap().clear();
                auth::delete_token_from_disk(&app);
                let tray_guard = state.tray.lock().unwrap();
                if let Some(tray) = tray_guard.as_ref() {
                    if let Ok(m) = menu::build_auth_menu(&app) {
                        let _ = tray.set_menu(Some(m));
                    }
                }
                let _ = app.emit("auth-cleared", ());
            });
        }

        _ => {}
    }
}
