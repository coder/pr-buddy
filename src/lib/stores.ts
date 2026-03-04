import { writable } from "svelte/store";
import Rocket from "@lucide/svelte/icons/rocket";
import XCircle from "@lucide/svelte/icons/x-circle";
import RotateCcw from "@lucide/svelte/icons/rotate-ccw";
import Eye from "@lucide/svelte/icons/eye";
import CircleDot from "@lucide/svelte/icons/circle-dot";
import Loader from "@lucide/svelte/icons/loader";
import CheckCircle from "@lucide/svelte/icons/check-circle";
import FileEdit from "@lucide/svelte/icons/file-edit";
import GitMerge from "@lucide/svelte/icons/git-merge";
import UserCheck from "@lucide/svelte/icons/user-check";
import type { PullRequest, GitHubUser, PrSection } from "./types";

export const prs = writable<PullRequest[]>([]);
export const user = writable<GitHubUser | null>(null);
export const authenticated = writable(false);
export const loading = writable(true);
export const lastUpdated = writable<Date | null>(null);

export function groupPrs(allPrs: PullRequest[]): PrSection[] {
  const reviewRequested = allPrs.filter(pr => pr.is_review_requested);
  const myPrs = allPrs.filter(pr => !pr.is_review_requested);

  const drafts = myPrs.filter(pr => pr.state === "open" && pr.is_draft);
  const nonDraftOpen = myPrs.filter(pr => pr.state === "open" && !pr.is_draft);

  const sections: PrSection[] = [
    {
      title: "Needs Your Review",
      icon: UserCheck,
      prs: reviewRequested.filter(pr =>
        pr.state === "open" && pr.check_status === "success"
      ),
    },
    {
      title: "In Merge Queue",
      icon: Rocket,
      prs: nonDraftOpen.filter(pr => pr.merge_queue_info != null),
    },
    {
      title: "Checks Failing",
      icon: XCircle,
      prs: nonDraftOpen.filter(pr =>
        pr.merge_queue_info == null &&
        (pr.check_status === "failure" || pr.check_status === "error")
      ),
    },
    {
      title: "Changes Requested",
      icon: RotateCcw,
      prs: nonDraftOpen.filter(pr =>
        pr.merge_queue_info == null &&
        pr.check_status !== "failure" && pr.check_status !== "error" &&
        pr.review_decision === "CHANGES_REQUESTED"
      ),
    },
    {
      title: "Mergeable",
      icon: CircleDot,
      prs: nonDraftOpen.filter(pr =>
        pr.merge_queue_info == null &&
        pr.check_status === "success" &&
        pr.review_decision !== "CHANGES_REQUESTED" &&
        pr.review_decision !== "APPROVED" &&
        (pr.review_decision === "REVIEW_REQUIRED" || pr.review_decision == null)
      ),
    },
    {
      title: "Checks Running",
      icon: Loader,
      prs: nonDraftOpen.filter(pr =>
        pr.merge_queue_info == null &&
        pr.check_status === "pending" &&
        pr.review_decision !== "CHANGES_REQUESTED" &&
        pr.review_decision !== "APPROVED"
      ),
    },
    {
      title: "Waiting for Review",
      icon: Eye,
      prs: nonDraftOpen.filter(pr =>
        pr.merge_queue_info == null &&
        pr.check_status === "none" &&
        pr.review_decision !== "CHANGES_REQUESTED" &&
        pr.review_decision !== "APPROVED" &&
        (pr.review_decision === "REVIEW_REQUIRED" || pr.review_decision == null)
      ),
    },
    {
      title: "Approved",
      icon: CheckCircle,
      prs: nonDraftOpen.filter(pr =>
        pr.merge_queue_info == null &&
        pr.check_status !== "failure" && pr.check_status !== "error" &&
        pr.review_decision === "APPROVED"
      ),
    },
    {
      title: "Draft",
      icon: FileEdit,
      prs: drafts,
    },
    {
      title: "Recently Merged",
      icon: GitMerge,
      prs: myPrs.filter(pr => pr.state === "merged"),
    },
  ];

  return sections.filter(s => s.prs.length > 0);
}
