use std::collections::HashMap;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::auth;

use crate::github::fetch_pull_requests;
use crate::models::{CheckStatus, PullRequest};
use crate::notifications::{diff_pr_states, send_notification};
use crate::state::AppState;

const POLL_INTERVAL_ACTIVE: u64 = 30;
const POLL_INTERVAL_IDLE: u64 = 120;

fn has_active_items(prs: &[PullRequest]) -> bool {
    prs.iter().any(|pr| {
        pr.check_status == CheckStatus::Pending || pr.merge_queue_info.is_some()
    })
}

pub fn start_polling(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            let token = {
                let state = app_handle.state::<AppState>();
                let t = state.token.lock().unwrap();
                t.clone()
            };

            if let Some(token) = token {
                match fetch_pull_requests(&token).await {
                    Ok(new_prs) => {
                        let events = {
                            let state = app_handle.state::<AppState>();
                            let previous = state.previous_prs.lock().unwrap();
                            diff_pr_states(&previous, &new_prs)
                        };

                        // Send notifications for state changes
                        for event in &events {
                            send_notification(&app_handle, event);
                        }

                        // Update stored state
                        {
                            let state = app_handle.state::<AppState>();
                            let mut previous = state.previous_prs.lock().unwrap();
                            *previous = new_prs
                                .iter()
                                .map(|pr| (pr.id.clone(), pr.clone()))
                                .collect::<HashMap<_, _>>();

                            let mut stored_prs = state.prs.lock().unwrap();
                            *stored_prs = new_prs.clone();

                            let mut last_poll = state.last_poll.lock().unwrap();
                            *last_poll = Some(chrono::Utc::now());
                        }

                        // Rebuild tray menu with updated PRs
                        {
                            let state = app_handle.state::<AppState>();
                            let tray_guard = state.tray.lock().unwrap();
                            if let Some(tray) = tray_guard.as_ref() {
                                if let Ok(new_menu) = crate::menu::build_pr_menu(&app_handle, &new_prs) {
                                    let _ = tray.set_menu(Some(new_menu));
                                }
                            }
                        }

                        // Emit to frontend webview
                        let _ = app_handle.emit("prs-updated", &new_prs);

                        // Adaptive sleep
                        let interval = if has_active_items(&new_prs) {
                            POLL_INTERVAL_ACTIVE
                        } else {
                            POLL_INTERVAL_IDLE
                        };
                        tokio::time::sleep(Duration::from_secs(interval)).await;
                    }
                    Err(_) => {
                        // Check if this is an auth failure (revoked token)
                        match crate::github::validate_token(&token).await {
                            Some(false) => {
                                // Token is confirmed invalid — clear session
                                eprintln!("[poller] Token is invalid, clearing session");
                                let state = app_handle.state::<AppState>();
                                *state.token.lock().unwrap() = None;
                                auth::delete_token_from_disk(&app_handle);
                                {
                                    let tray_guard = state.tray.lock().unwrap();
                                    if let Some(tray) = tray_guard.as_ref() {
                                        if let Ok(m) = crate::menu::build_auth_menu(&app_handle) {
                                            let _ = tray.set_menu(Some(m));
                                        }
                                    }
                                }
                                let _ = app_handle.emit("auth-cleared", ());
                            }
                            _ => {
                                // Network/transient error — keep token, retry later
                            }
                        }
                        tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_IDLE)).await;
                    }
                }
            } else {
                // Not authenticated, check less frequently
                tokio::time::sleep(Duration::from_secs(POLL_INTERVAL_IDLE)).await;
            }
        }
    });
}
