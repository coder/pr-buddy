use tauri::AppHandle;
use tauri::menu::{CheckMenuItem, IconMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri_plugin_autostart::ManagerExt;

use crate::avatars::AvatarCache;
use crate::models::{CheckStatus, PrState, PullRequest};

struct PrSection {
    title: String,
    prs: Vec<PullRequest>,
}

/// Port of src/lib/stores.ts groupPrs() — same 7 sections, same filter logic
fn group_prs(all_prs: &[PullRequest]) -> Vec<PrSection> {
    let drafts: Vec<_> = all_prs
        .iter()
        .filter(|pr| pr.state == PrState::Open && pr.is_draft)
        .cloned()
        .collect();
    let non_draft_open: Vec<_> = all_prs
        .iter()
        .filter(|pr| pr.state == PrState::Open && !pr.is_draft)
        .cloned()
        .collect();

    vec![
        PrSection {
            title: "In Merge Queue".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| pr.merge_queue_info.is_some())
                .cloned()
                .collect(),
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
        },
        PrSection {
            title: "Waiting for Review".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && pr.check_status != CheckStatus::Failure
                        && pr.check_status != CheckStatus::Error
                        && pr.review_decision.as_deref() != Some("CHANGES_REQUESTED")
                        && pr.review_decision.as_deref() != Some("APPROVED")
                })
                .cloned()
                .collect(),
        },
        PrSection {
            title: "Approved".into(),
            prs: non_draft_open
                .iter()
                .filter(|pr| {
                    pr.merge_queue_info.is_none()
                        && pr.check_status != CheckStatus::Failure
                        && pr.check_status != CheckStatus::Error
                        && pr.review_decision.as_deref() == Some("APPROVED")
                })
                .cloned()
                .collect(),
        },
        PrSection {
            title: "Draft".into(),
            prs: drafts,
        },
        PrSection {
            title: "Recently Merged".into(),
            prs: all_prs
                .iter()
                .filter(|pr| pr.state == PrState::Merged)
                .cloned()
                .collect(),
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
            let mut label = format!(
                "  {} #{} — {}",
                pr.repository,
                pr.number,
                truncate(&pr.title, 32),
            );
            if pr.comment_count > 0 {
                label.push_str(&format!("  💬{}", pr.comment_count));
            }
            label.push_str(&format!("  {}", age));

            let icon = avatar_cache.get_image(&pr.author_login);
            let item = IconMenuItem::with_id(
                app,
                &format!("pr_{}", pr.id),
                &label,
                true,
                icon,
                None::<&str>,
            )?;
            menu.append(&item)?;
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
    let check_updates =
        MenuItem::with_id(app, "check_updates", "Check for Updates", true, None::<&str>)?;
    menu.append(&check_updates)?;
    let autostart_enabled = app.autolaunch().is_enabled().unwrap_or(false);
    let autostart_toggle = CheckMenuItem::with_id(
        app,
        "autostart_toggle",
        "Launch at login",
        true,
        autostart_enabled,
        None::<&str>,
    )?;
    menu.append(&autostart_toggle)?;
    let settings =
        MenuItem::with_id(app, "settings", "Settings...", true, None::<&str>)?;
    menu.append(&settings)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    menu.append(&sep2)?;
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
    let mut chars = s.chars();
    let truncated: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        format!("{}…", truncated)
    } else {
        truncated
    }
}
