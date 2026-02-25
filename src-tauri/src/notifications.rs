use std::collections::HashMap;

use tauri::AppHandle;
use tauri_plugin_notification::NotificationExt;

use crate::models::{CheckStatus, PrEvent, PrState, PullRequest};

pub fn diff_pr_states(
    old: &HashMap<String, PullRequest>,
    new: &[PullRequest],
) -> Vec<PrEvent> {
    let mut events = Vec::new();

    for new_pr in new {
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

pub fn send_notification(app: &AppHandle, event: &PrEvent) {
    let (title, body) = match event {
        PrEvent::ChecksFailed(pr) => (
            format!("❌ Checks Failed"),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
        PrEvent::RemovedFromMergeQueue(pr) => (
            format!("🚫 Removed from Merge Queue"),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
        PrEvent::Merged(pr) => (
            format!("🎉 PR Merged"),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
        PrEvent::ChecksPassed(pr) => (
            format!("✅ Checks Passed"),
            format!("{} #{} — {}", pr.repository, pr.number, pr.title),
        ),
    };

    let _ = app
        .notification()
        .builder()
        .title(&title)
        .body(&body)
        .show();
}
