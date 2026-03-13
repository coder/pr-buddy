import { render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeAll, describe, expect, it, vi } from "vitest";

// @ts-expect-error test helpers are exposed by the aliased mock module
import { __setInvokeHandler, __resetInvokeMock } from "@tauri-apps/api/core";
// @ts-expect-error test helpers are exposed by the aliased mock module
import { __triggerEvent, __resetListeners } from "@tauri-apps/api/event";


beforeAll(() => {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn().mockImplementation(() => ({
      matches: false,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
    })),
  });
});
afterEach(() => {
  __resetInvokeMock();
  __resetListeners();
});

function setupInvokeDefaults() {
  __setInvokeHandler((cmd: string) => {
    switch (cmd) {
      case "is_authenticated_cmd":
        return true;
      case "refresh_prs_cmd":
      case "get_pull_requests_cmd":
        return [
          {
            id: "PR_1",
            number: 42,
            title: "feat: merged PR",
            url: "https://github.com/test/repo/pull/42",
            state: "merged",
            repository: "repo",
            owner: "test",
            head_ref: "feat-x",
            base_ref: "main",
            check_status: "success",
            is_draft: false,
            labels: [],
            merge_queue_info: null,
            created_at: "2025-01-01T00:00:00Z",
            updated_at: "2025-01-01T00:00:00Z",
            review_decision: null,
            additions: 10,
            deletions: 3,
            comment_count: 0,
            author_login: "testuser",
            author_avatar_url: "https://avatars.githubusercontent.com/u/1?v=4",
            is_review_requested: false,
          },
          {
            id: "PR_2",
            number: 43,
            title: "feat: open PR",
            url: "https://github.com/test/repo/pull/43",
            state: "open",
            repository: "repo",
            owner: "test",
            head_ref: "feat-y",
            base_ref: "main",
            check_status: "success",
            is_draft: false,
            labels: [],
            merge_queue_info: null,
            created_at: "2025-01-01T00:00:00Z",
            updated_at: "2025-01-01T00:00:00Z",
            review_decision: "APPROVED",
            additions: 5,
            deletions: 1,
            comment_count: 2,
            author_login: "testuser",
            author_avatar_url: "https://avatars.githubusercontent.com/u/1?v=4",
            is_review_requested: false,
          },
        ];
      case "get_user_info_cmd":
        return {
          login: "testuser",
          avatar_url: "https://avatars.githubusercontent.com/u/1?v=4",
          name: "Test User",
        };
      case "get_settings_cmd":
        return {
          notify_checks_failed: true,
          notify_checks_passed: true,
          notify_merged: true,
          notify_removed_from_queue: true,
          hidden_repos: [],
        };
      default:
        return null;
    }
  });
}

describe("App — show-merged event", () => {
  it("focuses the Recently Merged section when show-merged fires", async () => {
    setupInvokeDefaults();

    const { default: App } = await import("./App.svelte");
    render(App);

    await waitFor(() => {
      expect(screen.getByText("Recently Merged")).toBeTruthy();
    });

    expect(screen.getByText("Approved")).toBeTruthy();
    expect(screen.queryByText("← All PRs")).toBeFalsy();

    __triggerEvent("show-merged");

    await waitFor(() => {
      expect(screen.getByText("← All PRs")).toBeTruthy();
    });

    expect(screen.getByText("Recently Merged")).toBeTruthy();
    expect(screen.queryByText("Approved")).toBeFalsy();
  });
});
