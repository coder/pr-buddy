use std::collections::HashMap;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};

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

                        // Emit update event to frontend
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
