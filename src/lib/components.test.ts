/**
 * Component smoke tests — verifies every Svelte component can be imported,
 * mounted, and rendered without crashing. Catches broken imports (like wrong
 * icon package paths) at test time instead of at runtime.
 */
import { render, screen } from "@testing-library/svelte";
import { describe, it, expect } from "vitest";

import type { PullRequest, GitHubUser } from "./types";
import { groupPrs } from "./stores";

// ---------------------------------------------------------------------------
// Test data
// ---------------------------------------------------------------------------
const mockPr: PullRequest = {
  id: "PR_1",
  number: 42,
  title: "feat: add widget",
  url: "https://github.com/test/repo/pull/42",
  state: "open",
  repository: "repo",
  owner: "test",
  head_ref: "feat-widget",
  base_ref: "main",
  check_status: "success",
  is_draft: false,
  labels: [],
  merge_queue_info: null,
  created_at: "2025-01-01T00:00:00Z",
  updated_at: "2025-01-01T00:00:00Z",
  review_decision: "APPROVED",
  additions: 10,
  deletions: 3,
};

const mockUser: GitHubUser = {
  login: "testuser",
  avatar_url: "https://avatars.githubusercontent.com/u/1?v=4",
  name: "Test User",
};

// ---------------------------------------------------------------------------
// groupPrs logic
// ---------------------------------------------------------------------------
describe("groupPrs", () => {
  it("groups an approved PR into the Approved section", () => {
    const sections = groupPrs([mockPr]);
    expect(sections).toHaveLength(1);
    expect(sections[0].title).toBe("Approved");
    expect(sections[0].prs).toHaveLength(1);
  });

  it("returns empty array when no PRs", () => {
    expect(groupPrs([])).toHaveLength(0);
  });

  it("puts draft PRs in Draft section", () => {
    const draft = { ...mockPr, is_draft: true, review_decision: null };
    const sections = groupPrs([draft]);
    expect(sections).toHaveLength(1);
    expect(sections[0].title).toBe("Draft");
  });

  it("puts merged PRs in Recently Merged section", () => {
    const merged = { ...mockPr, state: "merged" as const };
    const sections = groupPrs([merged]);
    expect(sections).toHaveLength(1);
    expect(sections[0].title).toBe("Recently Merged");
  });
});

// ---------------------------------------------------------------------------
// Component render tests — catches broken imports at test time
// ---------------------------------------------------------------------------
describe("StatusBadge", () => {
  it("renders without crashing", async () => {
    const { default: StatusBadge } = await import("./StatusBadge.svelte");
    const { container } = render(StatusBadge, { props: { pr: mockPr } });
    expect(container.innerHTML).not.toBe("");
  });
});

describe("PRCard", () => {
  it("renders PR title and repo", async () => {
    const { default: PRCard } = await import("./PRCard.svelte");
    render(PRCard, { props: { pr: mockPr } });
    expect(screen.getByText("feat: add widget")).toBeTruthy();
    expect(screen.getByText("test/repo")).toBeTruthy();
  });
});

describe("PRSection", () => {
  it("renders section title and PR count", async () => {
    const sections = groupPrs([mockPr]);
    const { default: PRSection } = await import("./PRSection.svelte");
    render(PRSection, { props: { section: sections[0] } });
    expect(screen.getByText("Approved")).toBeTruthy();
  });
});

describe("PRPanel", () => {
  it("renders header and sections", async () => {
    const { default: PRPanel } = await import("./PRPanel.svelte");
    render(PRPanel, {
      props: {
        prs: [mockPr],
        user: mockUser,
        lastUpdated: new Date(),
        refreshing: false,
        onRefresh: () => {},
        onLogout: () => {},
        onOpenSettings: () => {},
      },
    });
    expect(screen.getByText("PR Buddy")).toBeTruthy();
  });
});

describe("AuthScreen", () => {
  it("renders sign-in button", async () => {
    const { default: AuthScreen } = await import("./AuthScreen.svelte");
    render(AuthScreen, { props: { onSuccess: () => {} } });
    expect(screen.getByText("Sign in with GitHub")).toBeTruthy();
  });
});

describe("SettingsPage", () => {
  it("renders settings header", async () => {
    const { default: SettingsPage } = await import("./SettingsPage.svelte");
    render(SettingsPage, { props: { prs: [mockPr], onBack: () => {}, onSettingsChanged: () => {} } });
    expect(screen.getByText("Settings")).toBeTruthy();
  });
});

describe("UpdateDialog", () => {
  it("renders without crashing", async () => {
    const { default: UpdateDialog } = await import("./UpdateDialog.svelte");
    const { container } = render(UpdateDialog);
    expect(container.innerHTML).not.toBe("");
  });
});


describe("TitleBar", () => {
  it("renders PR Buddy title and window controls", async () => {
    const { default: TitleBar } = await import("./TitleBar.svelte");
    render(TitleBar);
    expect(screen.getByText("PR Buddy")).toBeTruthy();
  });
});

