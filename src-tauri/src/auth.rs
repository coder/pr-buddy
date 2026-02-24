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

    let response = client
        .post("https://github.com/login/device/code")
        .header("Accept", "application/json")
        .form(&[("client_id", &client_id), ("scope", &"repo read:user".to_string())])
        .send()
        .await
        .map_err(|e| AuthError {
            message: format!("Failed to start device flow: {}", e),
        })?;

    response.json::<DeviceCodeResponse>().await.map_err(|e| AuthError {
        message: format!("Failed to parse device code response: {}", e),
    })
}

#[tauri::command]
pub async fn poll_for_token_cmd(
    device_code: String,
    state: State<'_, AppState>,
) -> Result<bool, AuthError> {
    let client = Client::new();
    let client_id = get_client_id();

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
        .map_err(|e| AuthError {
            message: format!("Failed to poll for token: {}", e),
        })?;

    let token_response: TokenResponse = response.json().await.map_err(|e| AuthError {
        message: format!("Failed to parse token response: {}", e),
    })?;

    if let Some(token) = token_response.access_token {
        let mut stored_token = state.token.lock().unwrap();
        *stored_token = Some(token);
        Ok(true)
    } else if let Some(error) = token_response.error {
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
