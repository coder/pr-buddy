use std::collections::HashMap;
use std::sync::Mutex;

use chrono::{DateTime, Utc};
use tauri::tray::TrayIcon;

use crate::avatars::AvatarCache;
use crate::models::{GitHubUser, PullRequest};
use crate::settings::UserSettings;

pub struct AppState {
    pub prs: Mutex<Vec<PullRequest>>,
    pub previous_prs: Mutex<HashMap<String, PullRequest>>,
    pub token: Mutex<Option<String>>,
    pub user: Mutex<Option<GitHubUser>>,
    pub last_poll: Mutex<Option<DateTime<Utc>>>,
    pub tray: Mutex<Option<TrayIcon>>,
    pub settings: Mutex<UserSettings>,
    pub avatar_cache: AvatarCache,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            prs: Mutex::new(Vec::new()),
            previous_prs: Mutex::new(HashMap::new()),
            token: Mutex::new(None),
            user: Mutex::new(None),
            last_poll: Mutex::new(None),
            tray: Mutex::new(None),
            settings: Mutex::new(UserSettings::default()),
            avatar_cache: AvatarCache::new(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
