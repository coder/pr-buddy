use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrState {
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Pending,
    Success,
    Failure,
    Error,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MergeQueueInfo {
    pub state: String,
    pub position: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PullRequest {
    pub id: String,
    pub number: u64,
    pub title: String,
    pub url: String,
    pub state: PrState,
    pub repository: String,
    pub owner: String,
    pub head_ref: String,
    pub base_ref: String,
    pub check_status: CheckStatus,
    pub is_draft: bool,
    pub labels: Vec<Label>,
    pub merge_queue_info: Option<MergeQueueInfo>,
    pub created_at: String,
    pub updated_at: String,
    pub review_decision: Option<String>,
    pub additions: u64,
    pub deletions: u64,
    pub comment_count: u64,
    pub author_login: String,
    pub author_avatar_url: String,
    pub is_review_requested: bool,
    pub merge_state_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubUser {
    pub login: String,
    pub avatar_url: String,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrEvent {
    ChecksFailed(PullRequest),
    RemovedFromMergeQueue(PullRequest),
    Merged(PullRequest),
    ChecksPassed(PullRequest),
}
