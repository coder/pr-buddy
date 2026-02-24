export type PrState = "open" | "closed" | "merged";
export type CheckStatus = "pending" | "success" | "failure" | "error" | "none";

export interface MergeQueueInfo {
  state: string;
  position: number | null;
}

export interface Label {
  name: string;
  color: string;
}

export interface PullRequest {
  id: string;
  number: number;
  title: string;
  url: string;
  state: PrState;
  repository: string;
  owner: string;
  head_ref: string;
  base_ref: string;
  check_status: CheckStatus;
  is_draft: boolean;
  labels: Label[];
  merge_queue_info: MergeQueueInfo | null;
  created_at: string;
  updated_at: string;
  review_decision: string | null;
  additions: number;
  deletions: number;
}

export interface GitHubUser {
  login: string;
  avatar_url: string;
  name: string | null;
}

export interface DeviceCodeResponse {
  device_code: string;
  user_code: string;
  verification_uri: string;
  expires_in: number;
  interval: number;
}

export interface PrSection {
  title: string;
  icon: string;
  prs: PullRequest[];
}
