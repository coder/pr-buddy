use std::collections::HashMap;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

use crate::auth;

use crate::github::fetch_pull_requests;
use crate::models::{CheckStatus, PrEvent, PullRequest};
use crate::notifications::{diff_pr_states, send_notification};
use crate::state::AppState;

const POLL_INTERVAL_ACTIVE: u64 = 30;
const POLL_INTERVAL_IDLE: u64 = 120;

/// Check for updates roughly every 4 hours.
/// At the idle interval of 120s, that's ~120 poll cycles.
const UPDATE_CHECK_EVERY_N_POLLS: u32 = 120;

fn has_active_items(prs: &[PullRequest]) -> bool {
    prs.iter().any(|pr| {
        pr.check_status == CheckStatus::Pending || pr.merge_queue_info.is_some()
    })
}

pub fn start_polling(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        // Start at the threshold so the first iteration triggers an update check
        let mut update_poll_counter: u32 = UPDATE_CHECK_EVERY_N_POLLS;

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

                        // Send notifications for state changes (gated by user settings)
                        {
                            let settings = app_handle.state::<AppState>().settings.lock().unwrap().clone();
                            for event in &events {
                                let should_notify = match event {
                                    PrEvent::ChecksFailed(_) => settings.notify_checks_failed,
                                    PrEvent::ChecksPassed(_) => settings.notify_checks_passed,
                                    PrEvent::Merged(_) => settings.notify_merged,
                                    PrEvent::RemovedFromMergeQueue(_) => settings.notify_removed_from_queue,
                                };
                                if should_notify {
                                    send_notification(&app_handle, event);
                                }
                            }
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

                        // Fetch avatars for any new authors
                        app_handle
                            .state::<AppState>()
                            .avatar_cache
                            .fetch_missing(&new_prs)
                            .await;
                        // Rebuild tray menu with updated PRs
                        {
                            let state = app_handle.state::<AppState>();
                            let tray_guard = state.tray.lock().unwrap();
                            if let Some(tray) = tray_guard.as_ref() {
                                if let Ok(new_menu) = crate::menu::build_pr_menu(
                                    &app_handle,
                                    &new_prs,
                                    &state.avatar_cache,
                                ) {
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
                                // Token is confirmed invalid — clear session,
                                // but only if it hasn't been replaced by a fresh login.
                                let state = app_handle.state::<AppState>();
                                let mut current = state.token.lock().unwrap();
                                if current.as_deref() == Some(token.as_str()) {
                                    eprintln!("[poller] Token is invalid, clearing session");
                                    *current = None;
                                    drop(current);
                                    auth::delete_token_from_disk(&app_handle);
                                    // Clear cached state to avoid stale data on re-login
                                    state.prs.lock().unwrap().clear();
                                    state.previous_prs.lock().unwrap().clear();
                                    *state.user.lock().unwrap() = None;
                                    {
                                        let tray_guard = state.tray.lock().unwrap();
                                        if let Some(tray) = tray_guard.as_ref() {
                                            if let Ok(m) = crate::menu::build_auth_menu(&app_handle) {
                                                let _ = tray.set_menu(Some(m));
                                            }
                                        }
                                    }
                                    let _ = app_handle.emit("auth-cleared", ());
                                } else {
                                    eprintln!("[poller] Token changed during validation, keeping new session");
                                }
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

            // Periodic update check (~every 4 hours)
            update_poll_counter += 1;
            if update_poll_counter >= UPDATE_CHECK_EVERY_N_POLLS {
                update_poll_counter = 0;
                let _ = crate::updater::check_for_update(&app_handle).await;
            }
        }
    });
}
