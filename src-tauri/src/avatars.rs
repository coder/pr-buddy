use std::collections::HashMap;
use std::sync::Mutex;

use crate::models::PullRequest;

const AVATAR_SIZE: u32 = 20;

struct CachedAvatar {
    rgba: Vec<u8>,
    size: u32,
}

pub struct AvatarCache {
    cache: Mutex<HashMap<String, CachedAvatar>>,
}

impl AvatarCache {
    pub fn new() -> Self {
        Self {
            cache: Mutex::new(HashMap::new()),
        }
    }

    /// Return a Tauri Image for the given login, or None if not cached.
    pub fn get_image(&self, login: &str) -> Option<tauri::image::Image<'static>> {
        let cache = self.cache.lock().unwrap();
        cache
            .get(login)
            .map(|av| tauri::image::Image::new_owned(av.rgba.clone(), av.size, av.size))
    }

    /// Download + process avatars for unique logins not already cached.
    pub async fn fetch_missing(&self, prs: &[PullRequest]) {
        let to_fetch: Vec<(String, String)> = {
            let cache = self.cache.lock().unwrap();
            prs.iter()
                .filter(|pr| {
                    !pr.author_avatar_url.is_empty() && !cache.contains_key(&pr.author_login)
                })
                .map(|pr| (pr.author_login.clone(), pr.author_avatar_url.clone()))
                .collect::<HashMap<_, _>>()
                .into_iter()
                .collect()
        };

        for (login, url) in to_fetch {
            if let Some(avatar) = download_and_process(&url).await {
                self.cache.lock().unwrap().insert(login, avatar);
            }
        }
    }
}

async fn download_and_process(url: &str) -> Option<CachedAvatar> {
    let sized_url = if url.contains('?') {
        format!("{}&s={}", url, AVATAR_SIZE)
    } else {
        format!("{}?s={}", url, AVATAR_SIZE)
    };
    let bytes = reqwest::get(&sized_url).await.ok()?.bytes().await.ok()?;
    let img = image::load_from_memory(&bytes).ok()?;
    let resized = img.resize_exact(
        AVATAR_SIZE,
        AVATAR_SIZE,
        image::imageops::FilterType::Lanczos3,
    );
    let mut rgba = resized.to_rgba8();

    // Apply circular mask
    let center = AVATAR_SIZE as f32 / 2.0;
    let radius = center;
    for y in 0..AVATAR_SIZE {
        for x in 0..AVATAR_SIZE {
            let dx = x as f32 - center + 0.5;
            let dy = y as f32 - center + 0.5;
            if dx * dx + dy * dy > radius * radius {
                rgba.get_pixel_mut(x, y).0[3] = 0;
            }
        }
    }

    Some(CachedAvatar {
        rgba: rgba.into_raw(),
        size: AVATAR_SIZE,
    })
}
