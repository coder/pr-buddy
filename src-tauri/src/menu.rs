use tauri::AppHandle;
use tauri::menu::{IconMenuItem, Menu, MenuItem, PredefinedMenuItem};

use crate::avatars::AvatarCache;
use crate::models::{CheckStatus, PrState, PullRequest};

struct PrSection {
    title: String,
    prs: Vec<PullRequest>,
    default_collapsed: bool,
}

/// Returns true when GitHub considers the PR actually mergeable (clean or has hooks).
fn is_actually_mergeable(pr: &PullRequest) -> bool {
    matches!(
        pr.merge_state_status.as_deref(),
        Some("CLEAN") | Some("HAS_HOOKS")
    )
}

/// Port of src/lib/stores.ts groupPrs() matching its section ordering.
fn group_prs(all_prs: &[PullRequest]) -> Vec<PrSection> {
    let review_requested: Vec<_> = all_prs
        .iter()
        .filter(|pr| pr.is_review_requested)
        .cloned()
        .collect();
    let my_prs: Vec<_> = all_prs
        .iter()
        .filter(|pr| !pr.is_review_requested)
        .cloned()
        .collect();

    let drafts: Vec<_> = my_prs
        .iter()
        .filter(|pr| pr.state == PrState::Open && pr.is_draft)
        .cloned()
        .collect();
    let non_draft_open: Vec<_> = my_prs
        .iter()
        .filter(|pr| pr.state == PrState::Open && !pr.is_draft)
        .cloned()
        .collect();

    vec![
        PrSection {
            title: "Needs Your Review".into(),
            prs: review_requested
                .iter()
                .filter(|pr| pr.state == PrState::Open && pr.check_status == CheckStatus::Success)
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "In Merge Queue".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| pr.merge_queue_info.is_some())
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "Checks Failing".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && (pr.check_status == CheckStatus::Failure
                            || pr.check_status == CheckStatus::Error)
                })
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "Changes Requested".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && pr.check_status != CheckStatus::Failure
                        && pr.check_status != CheckStatus::Error
                        && pr.review_decision.as_deref() == Some("CHANGES_REQUESTED")
                })
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "Mergeable".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && is_actually_mergeable(pr)
                        && pr.check_status != CheckStatus::Failure
                        && pr.check_status != CheckStatus::Error
                        && pr.review_decision.as_deref() != Some("CHANGES_REQUESTED")
                })
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "Checks Running".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && pr.check_status == CheckStatus::Pending
                        && pr.review_decision.as_deref() != Some("CHANGES_REQUESTED")
                        && pr.review_decision.as_deref() != Some("APPROVED")
                })
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "Waiting for Review".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && !is_actually_mergeable(pr)
                        && pr.check_status != CheckStatus::Failure
                        && pr.check_status != CheckStatus::Error
                        && pr.check_status != CheckStatus::Pending
                        && pr.review_decision.as_deref() != Some("CHANGES_REQUESTED")
                        && pr.review_decision.as_deref() != Some("APPROVED")
                })
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "Approved".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && !is_actually_mergeable(pr)
                        && pr.check_status != CheckStatus::Failure
                        && pr.check_status != CheckStatus::Error
                        && pr.review_decision.as_deref() == Some("APPROVED")
                })
                .cloned()
                .collect(),
            default_collapsed: false,
        },
        PrSection {
            title: "Draft".into(),
            prs: drafts,
            default_collapsed: false,
        },
        PrSection {
            title: "Recently Merged".into(),
            prs: my_prs
                .iter()
                .filter(|pr| pr.state == PrState::Merged)
                .cloned()
                .collect(),
            default_collapsed: true,
        },
    ]
}

/// Build the full PR menu with grouped sections
pub fn build_pr_menu(
    app: &AppHandle,
    prs: &[PullRequest],
    avatar_cache: &AvatarCache,
) -> tauri::Result<Menu<tauri::Wry>> {
    let sections = group_prs(prs);
    let menu = Menu::new(app)?;

    let mut first = true;
    for section in &sections {
        if section.prs.is_empty() {
            continue;
        }

        if !first {
            let sep = PredefinedMenuItem::separator(app)?;
            menu.append(&sep)?;
        }
        first = false;

        if section.default_collapsed {
            let item_text = format!("{} ({})", section.title, section.prs.len());
            let item = MenuItem::with_id(app, "open_panel", &item_text, true, None::<&str>)?;
            menu.append(&item)?;
        } else {
            // Section header (disabled)
            let header_text = format!("{} ({})", section.title, section.prs.len());
            let header = MenuItem::with_id(
                app,
                &format!("header_{}", section.title),
                &header_text,
                false,
                None::<&str>,
            )?;
            menu.append(&header)?;

            // PR items (max 5 per section)
            let show_count = section.prs.len().min(5);
            for pr in &section.prs[..show_count] {
                let age = time_ago(&pr.created_at);
                let label = format_pr_label(pr, &age);

                let icon = avatar_cache.get_image(&pr.author_login);
                let item_id = if section.title == "Checks Failing" {
                    format!("pr_checks_{}", pr.id)
                } else {
                    format!("pr_{}", pr.id)
                };
                let item = IconMenuItem::with_id(
                    app,
                    &item_id,
                    &label,
                    true,
                    icon,
                    None::<&str>,
                )?;
                menu.append(&item)?;
            }
        }

    }

    // If no sections had PRs, show empty state
    if first {
        let empty = MenuItem::with_id(app, "empty", "No pull requests", false, None::<&str>)?;
        menu.append(&empty)?;
    }

    // Footer
    let sep = PredefinedMenuItem::separator(app)?;
    menu.append(&sep)?;
    let see_all = MenuItem::with_id(app, "see_all", "See all on GitHub ↗", true, None::<&str>)?;
    menu.append(&see_all)?;
    let refresh = MenuItem::with_id(app, "refresh", "Refresh", true, None::<&str>)?;
    menu.append(&refresh)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    menu.append(&sep2)?;
    let check_updates =
        MenuItem::with_id(app, "check_updates", "Check for Updates", true, None::<&str>)?;
    menu.append(&check_updates)?;
    let settings = MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
    menu.append(&settings)?;
    let logout = MenuItem::with_id(app, "logout", "Logout", true, None::<&str>)?;
    menu.append(&logout)?;
    let quit = MenuItem::with_id(app, "quit", "Quit PR Buddy", true, None::<&str>)?;
    menu.append(&quit)?;

    Ok(menu)
}

/// Auth menu shown when not logged in
pub fn build_auth_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    let sign_in = MenuItem::with_id(app, "sign_in", "Sign in with GitHub", true, None::<&str>)?;
    menu.append(&sign_in)?;
    let sep = PredefinedMenuItem::separator(app)?;
    menu.append(&sep)?;
    let check_updates =
        MenuItem::with_id(app, "check_updates", "Check for Updates", true, None::<&str>)?;
    menu.append(&check_updates)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    menu.append(&sep2)?;
    let quit = MenuItem::with_id(app, "quit", "Quit PR Buddy", true, None::<&str>)?;
    menu.append(&quit)?;
    Ok(menu)
}

/// Auth-pending menu shown during device flow
pub fn build_auth_pending_menu(
    app: &AppHandle,
    user_code: &str,
) -> tauri::Result<Menu<tauri::Wry>> {
    let menu = Menu::new(app)?;
    let code_label = format!("Code: {} — copied!", user_code);
    let code_item = MenuItem::with_id(app, "code_display", &code_label, false, None::<&str>)?;
    menu.append(&code_item)?;
    let waiting = MenuItem::with_id(app, "waiting", "Waiting for authorization...", false, None::<&str>)?;
    menu.append(&waiting)?;
    let sep = PredefinedMenuItem::separator(app)?;
    menu.append(&sep)?;
    let quit = MenuItem::with_id(app, "quit", "Quit PR Buddy", true, None::<&str>)?;
    menu.append(&quit)?;
    Ok(menu)
}

/// Maximum characters for the left portion of a PR label (prefix + title).
/// The title is dynamically truncated to fit within this budget.
const MAX_LEFT_WIDTH: usize = 42;

/// Format a PR label with a tab stop so macOS right-aligns the suffix.
///
/// Native macOS menus treat `\t` as a right-aligned tab stop, so the suffix
/// (comment count + age) lines up regardless of proportional-font glyph widths.
fn format_pr_label(pr: &PullRequest, age: &str) -> String {
    let suffix = if pr.comment_count > 0 {
        format!("💬{}  {}", pr.comment_count, age)
    } else {
        age.to_string()
    };

    let prefix = format!("{} #{} — ", pr.repository, pr.number);
    let prefix_len = prefix.chars().count();
    let title_budget = MAX_LEFT_WIDTH.saturating_sub(prefix_len);

    let title = truncate(&pr.title, title_budget);

    format!("  {}{}\t{}", prefix, title, suffix)
}

fn time_ago(iso: &str) -> String {
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(iso) else {
        return String::new();
    };
    let secs = (chrono::Utc::now() - dt.to_utc()).num_seconds().max(0);
    if secs < 60 {
        return format!("{}s", secs);
    }
    if secs < 3600 {
        return format!("{}m", secs / 60);
    }
    if secs < 86400 {
        return format!("{}h", secs / 3600);
    }
    format!("{}d", secs / 86400)
}

fn truncate(s: &str, max: usize) -> String {
    if max == 0 {
        return String::new();
    }
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        // Reserve one char for the ellipsis so the result is exactly `max` chars.
        let trimmed: String = truncated.chars().take(max - 1).collect();
        format!("{}…", trimmed)
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::*;

    fn make_pr() -> PullRequest {
        PullRequest {
            id: "PR_1".into(),
            number: 1,
            title: "test".into(),
            url: "https://github.com/test/repo/pull/1".into(),
            state: PrState::Open,
            repository: "repo".into(),
            owner: "test".into(),
            head_ref: "feature".into(),
            base_ref: "main".into(),
            check_status: CheckStatus::Success,
            is_draft: false,
            labels: vec![],
            merge_queue_info: None,
            created_at: "2025-01-01T00:00:00Z".into(),
            updated_at: "2025-01-01T00:00:00Z".into(),
            review_decision: None,
            additions: 0,
            deletions: 0,
            comment_count: 0,
            author_login: "user".into(),
            author_avatar_url: "".into(),
            is_review_requested: false,
            merge_state_status: None,
        }
    }

    fn find_section<'a>(sections: &'a [PrSection], title: &str) -> Option<&'a PrSection> {
        sections.iter().find(|s| s.title == title)
    }

    #[test]
    fn blocked_review_required_not_in_mergeable() {
        let pr = PullRequest {
            review_decision: Some("REVIEW_REQUIRED".into()),
            merge_state_status: Some("BLOCKED".into()),
            check_status: CheckStatus::Success,
            ..make_pr()
        };
        let sections = group_prs(&[pr]);
        assert!(
            find_section(&sections, "Mergeable").is_none()
                || find_section(&sections, "Mergeable").unwrap().prs.is_empty(),
            "BLOCKED PR should not appear in Mergeable"
        );
        let waiting = find_section(&sections, "Waiting for Review")
            .expect("should be in Waiting for Review");
        assert_eq!(waiting.prs.len(), 1);
    }

    #[test]
    fn clean_pr_is_mergeable() {
        let pr = PullRequest {
            review_decision: None,
            merge_state_status: Some("CLEAN".into()),
            check_status: CheckStatus::Success,
            ..make_pr()
        };
        let sections = group_prs(&[pr]);
        let mergeable = find_section(&sections, "Mergeable").expect("should be in Mergeable");
        assert_eq!(mergeable.prs.len(), 1);
    }

    #[test]
    fn has_hooks_pr_is_mergeable() {
        let pr = PullRequest {
            review_decision: None,
            merge_state_status: Some("HAS_HOOKS".into()),
            check_status: CheckStatus::Success,
            ..make_pr()
        };
        let sections = group_prs(&[pr]);
        let mergeable = find_section(&sections, "Mergeable").expect("should be in Mergeable");
        assert_eq!(mergeable.prs.len(), 1);
    }

    #[test]
    fn approved_and_clean_is_mergeable() {
        let pr = PullRequest {
            review_decision: Some("APPROVED".into()),
            merge_state_status: Some("CLEAN".into()),
            check_status: CheckStatus::Success,
            ..make_pr()
        };
        let sections = group_prs(&[pr]);
        let mergeable = find_section(&sections, "Mergeable").expect("should be in Mergeable");
        assert_eq!(mergeable.prs.len(), 1);
        assert!(
            find_section(&sections, "Approved").is_none()
                || find_section(&sections, "Approved").unwrap().prs.is_empty()
        );
    }

    #[test]
    fn approved_but_blocked_stays_in_approved() {
        let pr = PullRequest {
            review_decision: Some("APPROVED".into()),
            merge_state_status: Some("BLOCKED".into()),
            check_status: CheckStatus::Success,
            ..make_pr()
        };
        let sections = group_prs(&[pr]);
        let approved = find_section(&sections, "Approved").expect("should be in Approved");
        assert_eq!(approved.prs.len(), 1);
        assert!(
            find_section(&sections, "Mergeable").is_none()
                || find_section(&sections, "Mergeable").unwrap().prs.is_empty()
        );
    }

    #[test]
    fn unknown_merge_state_not_mergeable() {
        let pr = PullRequest {
            review_decision: None,
            merge_state_status: None,
            check_status: CheckStatus::Success,
            ..make_pr()
        };
        let sections = group_prs(&[pr]);
        assert!(
            find_section(&sections, "Mergeable").is_none()
                || find_section(&sections, "Mergeable").unwrap().prs.is_empty(),
            "Unknown merge state should not be Mergeable"
        );
        let waiting = find_section(&sections, "Waiting for Review")
            .expect("should be in Waiting for Review");
        assert_eq!(waiting.prs.len(), 1);
    }
}
