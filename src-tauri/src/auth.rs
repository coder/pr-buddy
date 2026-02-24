use reqwest::Client;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::state::AppState;

const GITHUB_CLIENT_ID: &str = "Ov23liCVKFN3jOo9R7HS";

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceCodeResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u64,
    pub interval: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct TokenResponse {
    access_token: Option<String>,
    token_type: Option<String>,
    scope: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthError {
    pub message: String,
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

fn get_client_id() -> String {
    std::env::var("GITHUB_CLIENT_ID").unwrap_or_else(|_| GITHUB_CLIENT_ID.to_string())
}

#[tauri::command]
pub async fn start_device_flow_cmd() -> Result<DeviceCodeResponse, AuthError> {
    let client = Client::new();
    let client_id = get_client_id();
    eprintln!("[auth] Starting device flow with client_id={}", client_id);

    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", &client_id), ("scope", &"repo read:user".to_string())])
        .send()
        .await
        .map_err(|e| {
            eprintln!("[auth] Failed to start device flow: {}", e);
            AuthError {
                message: format!("Failed to start device flow: {}", e),
            }
        })?;

    let body = response.text().await.map_err(|e| {
        eprintln!("[auth] Failed to read device code response body: {}", e);
        AuthError {
            message: format!("Failed to read device code response: {}", e),
        }
    })?;
    eprintln!("[auth] Device code response: {}", body);

    serde_json::from_str::<DeviceCodeResponse>(&body).map_err(|e| {
        eprintln!("[auth] Failed to parse device code response: {}", e);
        AuthError {
            message: format!("Failed to parse device code response: {}", e),
        }
    })
}

#[tauri::command]
pub async fn poll_for_token_cmd(
    device_code: String,
    state: State<'_, AppState>,
) -> Result<bool, AuthError> {
    let client = Client::new();
    let client_id = get_client_id();
    eprintln!("[auth] Polling for token (device_code={}...)", &device_code[..8.min(device_code.len())]);

    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", &client_id),
            ("device_code", &device_code),
            ("grant_type", &"urn:ietf:params:oauth:grant-type:device_code".to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            eprintln!("[auth] Failed to poll for token: {}", e);
            AuthError {
                message: format!("Failed to poll for token: {}", e),
            }
        })?;

    let body = response.text().await.map_err(|e| {
        eprintln!("[auth] Failed to read token response body: {}", e);
        AuthError {
            message: format!("Failed to read token response: {}", e),
        }
    })?;
    eprintln!("[auth] Token response: {}", body);

    let token_response: TokenResponse = serde_json::from_str(&body).map_err(|e| {
        eprintln!("[auth] Failed to parse token response: {}", e);
        AuthError {
            message: format!("Failed to parse token response: {}", e),
        }
    })?;

    if let Some(ref token) = token_response.access_token {
        eprintln!("[auth] ✅ Token received ({}...)", &token[..8.min(token.len())]);
        let mut stored_token = state.token.lock().unwrap();
        *stored_token = Some(token.clone());
        Ok(true)
    } else if let Some(error) = token_response.error {
        eprintln!("[auth] GitHub response: error={}, desc={:?}", error, token_response.error_description);
        match error.as_str() {
            "authorization_pending" => Ok(false),
            "slow_down" => Ok(false),
            _ => Err(AuthError {
                message: token_response
                    .error_description
                    .unwrap_or_else(|| error.clone()),
            }),
        }
    } else {
        eprintln!("[auth] Unexpected response: no token and no error");
        Ok(false)
    }
}

#[tauri::command]
pub async fn logout_cmd(state: State<'_, AppState>) -> Result<(), AuthError> {
    let mut token = state.token.lock().unwrap();
    *token = None;
    let mut user = state.user.lock().unwrap();
    *user = None;
    let mut prs = state.prs.lock().unwrap();
    prs.clear();
    let mut previous_prs = state.previous_prs.lock().unwrap();
    previous_prs.clear();
    Ok(())
}

#[tauri::command]
pub async fn is_authenticated_cmd(state: State<'_, AppState>) -> Result<bool, AuthError> {
    let token = state.token.lock().unwrap();
    Ok(token.is_some())
}

/// Non-command version for direct Rust calls (e.g., from menu event handlers)
pub async fn start_device_flow() -> Result<DeviceCodeResponse, AuthError> {
    let client = Client::new();
    let client_id = get_client_id();
    eprintln!("[auth] Starting device flow with client_id={}", client_id);

    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", &client_id), ("scope", &"repo read:user".to_string())])
        .send()
        .await
        .map_err(|e| {
            eprintln!("[auth] Failed to start device flow: {}", e);
            AuthError {
                message: format!("Failed to start device flow: {}", e),
            }
        })?;

    let body = response.text().await.map_err(|e| {
        eprintln!("[auth] Failed to read device code response body: {}", e);
        AuthError {
            message: format!("Failed to read device code response: {}", e),
        }
    })?;
    eprintln!("[auth] Device code response: {}", body);

    serde_json::from_str::<DeviceCodeResponse>(&body).map_err(|e| {
        eprintln!("[auth] Failed to parse device code response: {}", e);
        AuthError {
            message: format!("Failed to parse device code response: {}", e),
        }
    })
}

/// Non-command version: takes &AppState directly instead of State<'_, AppState>
pub async fn poll_for_token(device_code: &str, state: &AppState) -> Result<bool, AuthError> {
    let client = Client::new();
    let client_id = get_client_id();
    eprintln!("[auth] Polling for token (device_code={}...)", &device_code[..8.min(device_code.len())]);

    let response = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", &client_id),
            ("device_code", &device_code.to_string()),
            ("grant_type", &"urn:ietf:params:oauth:grant-type:device_code".to_string()),
        ])
        .send()
        .await
        .map_err(|e| {
            eprintln!("[auth] Failed to poll for token: {}", e);
            AuthError {
                message: format!("Failed to poll for token: {}", e),
            }
        })?;

    let body = response.text().await.map_err(|e| {
        eprintln!("[auth] Failed to read token response body: {}", e);
        AuthError {
            message: format!("Failed to read token response: {}", e),
        }
    })?;
    eprintln!("[auth] Token response: {}", body);

    let token_response: TokenResponse = serde_json::from_str(&body).map_err(|e| {
        eprintln!("[auth] Failed to parse token response: {}", e);
        AuthError {
            message: format!("Failed to parse token response: {}", e),
        }
    })?;

    if let Some(ref token) = token_response.access_token {
        eprintln!("[auth] ✅ Token received ({}...)", &token[..8.min(token.len())]);
        let mut stored_token = state.token.lock().unwrap();
        *stored_token = Some(token.clone());
        Ok(true)
    } else if let Some(error) = token_response.error {
        eprintln!("[auth] GitHub response: error={}, desc={:?}", error, token_response.error_description);
        match error.as_str() {
            "authorization_pending" => Ok(false),
            "slow_down" => Ok(false),
            _ => Err(AuthError {
                message: token_response
                    .error_description
                    .unwrap_or_else(|| error.clone()),
            }),
        }
    } else {
        eprintln!("[auth] Unexpected response: no token and no error");
        Ok(false)
    }
}
