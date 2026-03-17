use std::collections::HashMap;

use tauri::AppHandle;

#[cfg(target_os = "macos")]
use crate::macos_notifications;
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

    #[cfg(target_os = "macos")]
    {
        macos_notifications::send_notification(app, &title, &body, &url);
    }

    #[cfg(not(target_os = "macos"))]
    {
        let app = app.clone();
        // Spawn a thread so the blocking notification call doesn't stall the async runtime.
        std::thread::spawn(move || {
            send_and_handle_click(&app, &title, &body, &url);
        });
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::models::{CheckStatus, Label, PrEvent, PrState, PullRequest};

    use super::diff_pr_states;

    fn sample_pull_request(id: &str) -> PullRequest {
        PullRequest {
            id: id.to_string(),
            number: 42,
            title: "Improve notifications".to_string(),
            url: format!("https://github.com/coder/pr-buddy/pull/{id}"),
            state: PrState::Open,
            repository: "coder/pr-buddy".to_string(),
            owner: "coder".to_string(),
            head_ref: "feature/notifications".to_string(),
            base_ref: "main".to_string(),
            check_status: CheckStatus::Pending,
            is_draft: false,
            labels: vec![Label {
                name: "enhancement".to_string(),
                color: "00ff00".to_string(),
            }],
            merge_queue_info: None,
            created_at: "2026-03-17T12:00:00Z".to_string(),
            updated_at: "2026-03-17T12:00:00Z".to_string(),
            review_decision: Some("APPROVED".to_string()),
            additions: 10,
            deletions: 2,
            comment_count: 3,
            author_login: "mike".to_string(),
            author_avatar_url: "https://avatars.githubusercontent.com/u/1".to_string(),
            is_review_requested: false,
        }
    }

    #[test]
    fn diff_pr_states_still_emits_checks_passed_when_status_flips_to_success() {
        let mut old_pr = sample_pull_request("checks-passed");
        old_pr.check_status = CheckStatus::Pending;

        let mut new_pr = old_pr.clone();
        new_pr.check_status = CheckStatus::Success;

        let old = HashMap::from([(old_pr.id.clone(), old_pr)]);
        let events = diff_pr_states(&old, &[new_pr.clone()]);

        assert!(matches!(
            events.as_slice(),
            [PrEvent::ChecksPassed(pr)] if pr == &new_pr
        ));
    }

    #[test]
    fn diff_pr_states_still_emits_merged_when_state_flips_to_merged() {
        let old_pr = sample_pull_request("merged");

        let mut new_pr = old_pr.clone();
        new_pr.state = PrState::Merged;

        let old = HashMap::from([(old_pr.id.clone(), old_pr)]);
        let events = diff_pr_states(&old, &[new_pr.clone()]);

        assert!(matches!(events.as_slice(), [PrEvent::Merged(pr)] if pr == &new_pr));
    }
}
