use std::collections::HashMap;

use tauri::AppHandle;

use crate::models::{CheckStatus, PrEvent, PrState, PullRequest};

pub fn diff_pr_states(
    old: &HashMap<String, PullRequest>,
    new: &[PullRequest],
) -> Vec<PrEvent> {
    let mut events = Vec::new();

    for new_pr in new {
        // Don't fire notifications for PRs we're just reviewing
        if new_pr.is_review_requested {
            continue;
        }

        if let Some(old_pr) = old.get(&new_pr.id) {
            // Check for checks failure
            if old_pr.check_status != CheckStatus::Failure
                && new_pr.check_status == CheckStatus::Failure
            {
                events.push(PrEvent::ChecksFailed(new_pr.clone()));
            }

            // Check for removal from merge queue
            if old_pr.merge_queue_info.is_some() && new_pr.merge_queue_info.is_none() {
                if new_pr.state != PrState::Merged {
                    events.push(PrEvent::RemovedFromMergeQueue(new_pr.clone()));
                }
            }

            // Check for merge
            if old_pr.state != PrState::Merged && new_pr.state == PrState::Merged {
                events.push(PrEvent::Merged(new_pr.clone()));
            }

            // Check for checks passing
            if old_pr.check_status != CheckStatus::Success
                && new_pr.check_status == CheckStatus::Success
            {
                events.push(PrEvent::ChecksPassed(new_pr.clone()));
            }
        }
    }

    events
}

/// Extract the PR URL from any event variant.
fn pr_url(event: &PrEvent) -> &str {
    match event {
        PrEvent::ChecksFailed(pr)
        | PrEvent::RemovedFromMergeQueue(pr)
        | PrEvent::Merged(pr)
        | PrEvent::ChecksPassed(pr) => &pr.url,
    }
}

pub fn send_notification(app: &AppHandle, event: &PrEvent) {
    let (title, body) = match event {
        PrEvent::ChecksFailed(pr) => (
            "❌ Checks Failed".to_string(),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
        PrEvent::RemovedFromMergeQueue(pr) => (
            "🚫 Removed from Merge Queue".to_string(),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
        PrEvent::Merged(pr) => (
            "🎉 PR Merged".to_string(),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
        PrEvent::ChecksPassed(pr) => (
            "✅ Checks Passed".to_string(),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
    };

    let url = pr_url(event).to_string();
    let app = app.clone();

    // Spawn a thread so the blocking notification call doesn't stall the async runtime.
    std::thread::spawn(move || {
        send_and_handle_click(&app, &title, &body, &url);
    });
}

/// macOS: fire-and-forget via tauri-plugin-notification (replaces mac-notification-sys
/// which caused 100% CPU from its busy-wait NSRunLoop polling).
#[cfg(target_os = "macos")]
fn send_and_handle_click(app: &AppHandle, title: &str, body: &str, _url: &str) {
    use tauri_plugin_notification::NotificationExt;
    if let Err(e) = app.notification().builder().title(title).body(body).show() {
        eprintln!("[notifications] Failed to show notification: {}", e);
    }
}

/// Linux: use notify-rust with wait_for_action (D-Bus ActionInvoked signal).
#[cfg(target_os = "linux")]
fn send_and_handle_click(app: &AppHandle, title: &str, body: &str, url: &str) {
    let mut n = notify_rust::Notification::new();
    n.summary(title).body(body).action("default", "Open PR");

    match n.show() {
        Ok(handle) => {
            let url = url.to_string();
            let app = app.clone();
            handle.wait_for_action(move |action| {
                if action == "default" {
                    use tauri_plugin_opener::OpenerExt;
                    let _ = app.opener().open_url(&url, None::<&str>);
                }
            });
        }
        Err(e) => {
            eprintln!("[notifications] Failed to show notification: {}", e);
        }
    }
}

/// Windows: fire-and-forget (no click handling yet).
#[cfg(target_os = "windows")]
fn send_and_handle_click(_app: &AppHandle, title: &str, body: &str, _url: &str) {
    let mut n = notify_rust::Notification::new();
    n.summary(title).body(body);

    if let Err(e) = n.show() {
        eprintln!("[notifications] Failed to show notification: {}", e);
    }
}
