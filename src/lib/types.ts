import type { Component, ComponentType, SvelteComponent } from "svelte";

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
  comment_count: number;
  author_login: string;
  author_avatar_url: string;
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

export interface UserSettings {
  notify_checks_failed: boolean;
  notify_checks_passed: boolean;
  notify_merged: boolean;
  notify_removed_from_queue: boolean;
  hidden_repos: string[];
}

export interface UpdateCheckResult {
  current_version: string;
  update_available: boolean;
  version: string | null;
  body: string | null;
}


export type IconComponent =
  | Component<{ size?: number | string; class?: string }>
  | ComponentType<SvelteComponent>;

export interface PrSection {
  title: string;
  icon: IconComponent;
  prs: PullRequest[];
}
