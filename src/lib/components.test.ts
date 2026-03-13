/**
 * Component smoke tests — verifies every Svelte component can be imported,
 * mounted, and rendered without crashing. Catches broken imports (like wrong
 * icon package paths) at test time instead of at runtime.
 */
import { fireEvent, render, screen } from "@testing-library/svelte";
import { openUrl } from "@tauri-apps/plugin-opener";
import { beforeEach, describe, expect, it, vi } from "vitest";

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
  comment_count: 5,
  author_login: "testuser",
  author_avatar_url: "https://avatars.githubusercontent.com/u/1?v=4",
  is_review_requested: false,
};

const mockUser: GitHubUser = {
  login: "testuser",
  avatar_url: "https://avatars.githubusercontent.com/u/1?v=4",
  name: "Test User",
};

beforeEach(() => {
  vi.clearAllMocks();
});

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

  it("puts review-requested checks-passed PRs in Needs Your Review section", () => {
    const reviewPr = { ...mockPr, is_review_requested: true, check_status: "success" as const, review_decision: null };
    const sections = groupPrs([reviewPr]);
    expect(sections).toHaveLength(1);
    expect(sections[0].title).toBe("Needs Your Review");
  });

  it("excludes review-requested PRs with failing checks from Needs Your Review", () => {
    const reviewPr = { ...mockPr, is_review_requested: true, check_status: "failure" as const };
    const sections = groupPrs([reviewPr]);
    expect(sections).toHaveLength(0);
  });

  it("does not mix review-requested PRs into authored sections", () => {
    const myPr = { ...mockPr, is_review_requested: false };
    const reviewPr = { ...mockPr, id: "PR_2", is_review_requested: true, check_status: "success" as const, review_decision: null };
    const sections = groupPrs([myPr, reviewPr]);
    expect(sections.find(s => s.title === "Needs Your Review")?.prs).toHaveLength(1);
    expect(sections.find(s => s.title === "Approved")?.prs).toHaveLength(1);
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
    const { container } = render(PRCard, { props: { pr: mockPr } });
    expect(screen.getByText("feat: add widget")).toBeTruthy();
    expect(screen.getByText("test/repo")).toBeTruthy();

    // Avatar renders
    const img = container.querySelector("img");
    expect(img).toBeTruthy();
    expect(img?.getAttribute("src")).toContain("avatars.githubusercontent.com");
  });

  it("opens a custom destination URL when the PR row is clicked", async () => {
    const { default: PRCard } = await import("./PRCard.svelte");
    const destinationUrl = `${mockPr.url}/checks`;
    const { container } = render(PRCard, { props: { pr: mockPr, destinationUrl } });

    await fireEvent.click(container.querySelector("button")!);

    expect(openUrl).toHaveBeenCalledWith(destinationUrl);
  });
});

describe("PRSection", () => {
  it("opens the PR checks page for Checks Failing rows", async () => {
    const failingPr = { ...mockPr, check_status: "failure" as const, review_decision: null };
    const sections = groupPrs([failingPr]);
    const checksFailing = sections.find(s => s.title === "Checks Failing");

    expect(checksFailing).toBeTruthy();

    const { default: PRSection } = await import("./PRSection.svelte");
    const { container } = render(PRSection, { props: { section: checksFailing! } });

    await fireEvent.click(container.querySelectorAll("button")[1]!);

    expect(openUrl).toHaveBeenCalledWith(`${mockPr.url}/checks`);
  });

  it("renders section title and PR count", async () => {
    const sections = groupPrs([mockPr]);
    const { default: PRSection } = await import("./PRSection.svelte");
    render(PRSection, { props: { section: sections[0] } });
    expect(screen.getByText("Approved")).toBeTruthy();
  });

  it("expands all PRs when expandAll is true (no pagination)", async () => {
    const manyPrs = Array.from({ length: 8 }, (_, i) => ({
      ...mockPr,
      id: `PR_expand_${i}`,
      number: 100 + i,
      title: `PR ${100 + i}`,
      state: "merged" as const,
    }));
    const { RECENTLY_MERGED_SECTION_TITLE } = await import("./constants");
    const section: import("./types").PrSection = {
      title: RECENTLY_MERGED_SECTION_TITLE,
      icon: (await import("@lucide/svelte/icons/git-merge")).default,
      prs: manyPrs,
      defaultCollapsed: true,
    };
    const { default: PRSection } = await import("./PRSection.svelte");
    render(PRSection, { props: { section, expandAll: true } });

    for (const pr of manyPrs) {
      expect(screen.getByText(pr.title)).toBeTruthy();
    }

    expect(screen.queryByText(/Show more/)).toBeFalsy();
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
    expect(screen.getByText("Approved")).toBeTruthy();
  });

  it("shows only the focused section with back control in focus mode", async () => {
    const { RECENTLY_MERGED_SECTION_TITLE } = await import("./constants");
    const approvedPr = { ...mockPr, id: "PR_A1", state: "open" as const, review_decision: "APPROVED" };
    const mergedPr = { ...mockPr, id: "PR_M1", state: "merged" as const };
    const clearFocusFn = vi.fn();

    const { default: PRPanel } = await import("./PRPanel.svelte");
    render(PRPanel, {
      props: {
        prs: [approvedPr, mergedPr],
        user: mockUser,
        lastUpdated: new Date(),
        refreshing: false,
        onRefresh: () => {},
        onLogout: () => {},
        onOpenSettings: () => {},
        focusSection: RECENTLY_MERGED_SECTION_TITLE,
        onClearFocus: clearFocusFn,
      },
    });

    expect(screen.getByText("Recently Merged")).toBeTruthy();
    expect(screen.queryByText("Approved")).toBeFalsy();

    const backBtn = screen.getByText("← All PRs");
    expect(backBtn).toBeTruthy();

    await fireEvent.click(backBtn);
    expect(clearFocusFn).toHaveBeenCalledOnce();
  });

  it("shows all sections without back control in normal mode", async () => {
    const approvedPr = { ...mockPr, id: "PR_A2", state: "open" as const, review_decision: "APPROVED" };
    const mergedPr = { ...mockPr, id: "PR_M2", state: "merged" as const };

    const { default: PRPanel } = await import("./PRPanel.svelte");
    render(PRPanel, {
      props: {
        prs: [approvedPr, mergedPr],
        user: mockUser,
        lastUpdated: new Date(),
        refreshing: false,
        onRefresh: () => {},
        onLogout: () => {},
        onOpenSettings: () => {},
      },
    });

    expect(screen.getByText("Approved")).toBeTruthy();
    expect(screen.getByText("Recently Merged")).toBeTruthy();
    expect(screen.queryByText("← All PRs")).toBeFalsy();
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

