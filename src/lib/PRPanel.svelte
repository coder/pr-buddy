<script lang="ts">
  import Inbox from "@lucide/svelte/icons/inbox";
  import PartyPopper from "@lucide/svelte/icons/party-popper";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import LogOut from "@lucide/svelte/icons/log-out";
  import Settings from "@lucide/svelte/icons/settings";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { PullRequest, GitHubUser } from "./types";
  import { groupPrs } from "./stores";
  import PRSection from "./PRSection.svelte";

  interface Props {
    prs: PullRequest[];
    user: GitHubUser | null;
    lastUpdated: Date | null;
    refreshing: boolean;
    onRefresh: () => void;
    onLogout: () => void;
    onOpenSettings: () => void;
    focusSection?: string | null;
    onClearFocus?: (() => void) | null;
  }
  let {
    prs,
    user,
    lastUpdated,
    refreshing,
    onRefresh,
    onLogout,
    onOpenSettings,
    focusSection = null,
    onClearFocus = null,
  }: Props = $props();

  let now = $state(new Date());
  $effect(() => {
    const id = setInterval(() => {
      now = new Date();
    }, 1000);
    return () => clearInterval(id);
  });

  let allSections = $derived(groupPrs(prs));
  let visibleSections = $derived(
    focusSection ? allSections.filter((s) => s.title === focusSection) : allSections,
  );

  function relativeTime(date: Date | null): string {
    if (!date) return "never";
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);
    if (diff < 5) return "just now";
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return `${Math.floor(diff / 3600)}h ago`;
  }
</script>

<div data-tauri-drag-region class="flex shrink-0 items-center justify-end gap-1.5 px-3 py-2">
  {#if user}
    <img src={user.avatar_url} alt={user.login} class="h-5 w-5 rounded-full" />
  {/if}
  <button
    onclick={onOpenSettings}
    class="rounded-md p-1 text-content-tertiary hover:bg-surface-hover hover:text-content"
    title="Settings"
  >
    <Settings size={13} />
  </button>
  <button
    onclick={onLogout}
    class="rounded-md p-1 text-content-tertiary hover:bg-surface-hover hover:text-content"
    title="Sign out"
  >
    <LogOut size={13} />
  </button>
</div>

{#if focusSection}
  <button
    onclick={() => onClearFocus?.()}
    class="px-3 py-1 text-[11px] font-medium text-accent hover:text-accent-hover"
  >
    ← All PRs
  </button>
{/if}

<div class="flex-1 overflow-y-auto min-h-0 scrollbar-thin">
  {#if prs.length === 0}
    <div class="flex h-full flex-col items-center justify-center gap-2 text-content-tertiary">
      <Inbox size={20} class="text-content-tertiary" />
      <p class="text-sm">No pull requests found</p>
      <p class="text-xs text-content-tertiary">Your PRs will appear here</p>
    </div>
  {:else if visibleSections.length === 0}
    <div class="flex h-full flex-col items-center justify-center gap-2 text-content-tertiary">
      <PartyPopper size={20} class="text-content-tertiary" />
      <p class="text-sm">All clear!</p>
    </div>
  {:else}
    <div class="space-y-3 px-3 py-1">
      {#each visibleSections as section (section.title)}
        <PRSection {section} expandAll={focusSection === section.title} />
      {/each}
    </div>
  {/if}
</div>

<div class="flex shrink-0 items-center justify-between px-3 py-2">
  <span class="text-[10px] text-content-tertiary">Updated {relativeTime(lastUpdated)}</span>
  <div class="flex items-center gap-1">
    <button
      onclick={() => openUrl("https://github.com/pulls")}
      class="px-1 py-1 text-[10px] text-content-tertiary hover:text-content-secondary"
    >
      See all
    </button>
    <button
      onclick={onRefresh}
      disabled={refreshing}
      title="Refresh"
      class="rounded-md p-1 text-[10px] text-content-tertiary hover:text-content-secondary disabled:opacity-50"
    >
      <RefreshCw size={11} class={refreshing ? "animate-spin" : ""} />
    </button>
  </div>
</div>
