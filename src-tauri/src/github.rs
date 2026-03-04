use reqwest::Client;
use serde_json::Value;
use tauri::State;

use crate::auth::AuthError;
use crate::models::{
    CheckStatus, GitHubUser, Label, MergeQueueInfo, PrState, PullRequest,
};
use crate::state::AppState;

const GRAPHQL_URL: &str = "https://api.github.com/graphql";

fn build_pr_query() -> String {
    let since = chrono::Utc::now() - chrono::Duration::days(14);
    let since_str = since.format("%Y-%m-%d").to_string();

    format!(
        r#"{{
  search(query: "is:pr author:@me updated:>={since}", type: ISSUE, first: 50) {{
    nodes {{
      ... on PullRequest {{
        id
        number
        title
        url
        state
        isDraft
        createdAt
        updatedAt
        reviewDecision
        additions
        deletions
        headRefName
        baseRefName
        repository {{
          name
          owner {{
            login
          }}
        }}
        labels(first: 10) {{
          nodes {{
            name
            color
          }}
        }}
        commits(last: 1) {{
          nodes {{
            commit {{
              statusCheckRollup {{
                state
              }}
            }}
          }}
        }}
        mergeQueueEntry {{
          state
          position
        }}
      }}
    }}
  }}
}}"#,
        since = since_str
    )
}

fn parse_check_status(pr_node: &Value) -> CheckStatus {
    pr_node
        .pointer("/commits/nodes/0/commit/statusCheckRollup/state")
        .and_then(|s| s.as_str())
        .map(|s| match s.to_uppercase().as_str() {
            "SUCCESS" => CheckStatus::Success,
            "PENDING" | "EXPECTED" => CheckStatus::Pending,
            "FAILURE" => CheckStatus::Failure,
            "ERROR" => CheckStatus::Error,
            _ => CheckStatus::None,
        })
        .unwrap_or(CheckStatus::None)
}

fn parse_pr_state(state: &str) -> PrState {
    match state.to_uppercase().as_str() {
        "OPEN" => PrState::Open,
        "CLOSED" => PrState::Closed,
        "MERGED" => PrState::Merged,
        _ => PrState::Open,
    }
}

fn parse_pr_node(node: &Value) -> Option<PullRequest> {
    Some(PullRequest {
        id: node.get("id")?.as_str()?.to_string(),
        number: node.get("number")?.as_u64()?,
        title: node.get("title")?.as_str()?.to_string(),
        url: node.get("url")?.as_str()?.to_string(),
        state: parse_pr_state(node.get("state")?.as_str()?),
        repository: node.pointer("/repository/name")?.as_str()?.to_string(),
        owner: node
            .pointer("/repository/owner/login")?
            .as_str()?
            .to_string(),
        head_ref: node.get("headRefName")?.as_str()?.to_string(),
        base_ref: node.get("baseRefName")?.as_str()?.to_string(),
        check_status: parse_check_status(node),
        is_draft: node.get("isDraft").and_then(|v| v.as_bool()).unwrap_or(false),
        labels: node
            .pointer("/labels/nodes")
            .and_then(|n| n.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|l| {
                        Some(Label {
                            name: l.get("name")?.as_str()?.to_string(),
                            color: l.get("color")?.as_str()?.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default(),
        merge_queue_info: node.get("mergeQueueEntry").and_then(|mq| {
            if mq.is_null() {
                return None;
            }
            Some(MergeQueueInfo {
                state: mq.get("state")?.as_str()?.to_string(),
                position: mq.get("position").and_then(|p| p.as_u64()).map(|p| p as u32),
            })
        }),
        created_at: node.get("createdAt")?.as_str()?.to_string(),
        updated_at: node.get("updatedAt")?.as_str()?.to_string(),
        review_decision: node
            .get("reviewDecision")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        additions: node.get("additions").and_then(|v| v.as_u64()).unwrap_or(0),
        deletions: node.get("deletions").and_then(|v| v.as_u64()).unwrap_or(0),
    })
}

pub async fn fetch_pull_requests(token: &str) -> Result<Vec<PullRequest>, AuthError> {
    let client = Client::new();
    let query = build_pr_query();

    let response = client
        .post(GRAPHQL_URL)
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "PR-Buddy")
        .json(&serde_json::json!({ "query": query }))
        .send()
        .await
        .map_err(|e| AuthError {
            message: format!("GraphQL request failed: {}", e),
        })?;

    let body: Value = response.json().await.map_err(|e| AuthError {
        message: format!("Failed to parse GraphQL response: {}", e),
    })?;

    let nodes = body
        .pointer("/data/search/nodes")
        .and_then(|n| n.as_array())
        .ok_or_else(|| AuthError {
            message: "Invalid GraphQL response structure".to_string(),
        })?;

    Ok(nodes.iter().filter_map(parse_pr_node).collect())
}

pub async fn fetch_user_info(token: &str) -> Result<GitHubUser, AuthError> {
    let client = Client::new();

    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "PR-Buddy")
        .send()
        .await
        .map_err(|e| AuthError {
            message: format!("Failed to fetch user info: {}", e),
        })?;

    response.json::<GitHubUser>().await.map_err(|e| AuthError {
        message: format!("Failed to parse user info: {}", e),
    })
}

/// Validate a token by making a lightweight API call.
/// Returns true only if the token is confirmed valid (200 OK).
/// Returns false only for a definitive 401 Unauthorized.
/// Returns None for network errors or any other status (403 can
/// mean rate-limiting/abuse, not just bad credentials).
pub async fn validate_token(token: &str) -> Option<bool> {
    let client = Client::new();
    let response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token))
        .header("User-Agent", "PR-Buddy")
        .send()
        .await
        .ok()?; // Network error → None (don't clear token)
    let status = response.status();
    if status.is_success() {
        Some(true)
    } else if status.as_u16() == 401 {
        Some(false)
    } else {
        // 403 (rate limit/abuse), 429, 5xx, etc. — don't invalidate
        eprintln!("[auth] Token validation got HTTP {}, treating as transient", status);
        None
    }
}

#[tauri::command]
pub async fn get_pull_requests_cmd(
    state: State<'_, AppState>,
) -> Result<Vec<PullRequest>, AuthError> {
    let prs = state.prs.lock().unwrap();
    Ok(prs.clone())
}

#[tauri::command]
pub async fn get_user_info_cmd(
    state: State<'_, AppState>,
) -> Result<Option<GitHubUser>, AuthError> {
    // Return cached user if available
    {
        let user = state.user.lock().unwrap();
        if user.is_some() {
            return Ok(user.clone());
        }
    }
    // Fetch from GitHub if we have a token but no cached user
    let token = {
        let t = state.token.lock().unwrap();
        t.clone()
    };
    if let Some(token) = token {
        let user_info = fetch_user_info(&token).await?;
        // Only cache and return if the token hasn't changed during the fetch
        // (guards against logout/account-switch racing with the request)
        let current_token = state.token.lock().unwrap().clone();
        if current_token.as_deref() == Some(token.as_str()) {
            let mut cached = state.user.lock().unwrap();
            *cached = Some(user_info.clone());
            Ok(Some(user_info))
        } else {
            Ok(None)
        }
    } else {
        Ok(None)
    }
}

#[tauri::command]
pub async fn refresh_prs_cmd(state: State<'_, AppState>) -> Result<Vec<PullRequest>, AuthError> {
    let token = {
        let t = state.token.lock().unwrap();
        t.clone().ok_or_else(|| AuthError {
            message: "Not authenticated".to_string(),
        })?
    };

    let prs = fetch_pull_requests(&token).await?;

    let mut stored_prs = state.prs.lock().unwrap();
    *stored_prs = prs.clone();

    Ok(prs)
}
