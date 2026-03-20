<script lang="ts">
  import Inbox from "@lucide/svelte/icons/inbox";
  import PartyPopper from "@lucide/svelte/icons/party-popper";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import ExternalLink from "@lucide/svelte/icons/external-link";
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
    const id = setInterval(() => { now = new Date(); }, 1000);
    return () => clearInterval(id);
  });

  let allSections = $derived(groupPrs(prs));
  let visibleSections = $derived(
    focusSection
      ? allSections.filter(s => s.title === focusSection)
      : allSections,
  );
  let totalPrs = $derived(prs.filter(pr => pr.state === "open").length);

  function relativeTime(date: Date | null): string {
    if (!date) return "never";
    const diff = Math.floor((now.getTime() - date.getTime()) / 1000);
    if (diff < 5) return "just now";
    if (diff < 60) return `${diff}s ago`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
    return `${Math.floor(diff / 3600)}h ago`;
  }
</script>

<!-- Header -->
<div class="flex items-center justify-between px-4 py-2.5 border-b border-border shrink-0">
  <div class="flex items-center gap-2">
    {#if totalPrs > 0}
      <span class="flex h-4 min-w-[16px] items-center justify-center rounded-full bg-accent/90 px-1 text-[9px] font-medium text-white">
        {totalPrs}
      </span>
    {/if}
  </div>
  <div class="flex items-center gap-2">
    {#if user}
      <img
        src={user.avatar_url}
        alt={user.login}
        class="w-6 h-6 rounded-full ring-1 ring-border"
      />
    {/if}
    <button
      onclick={onOpenSettings}
      class="text-content-tertiary hover:text-content transition-colors p-1 rounded hover:bg-surface-hover"
      title="Settings"
    >
      <Settings size={16} />
    </button>
    <button
      onclick={onLogout}
      class="text-content-tertiary hover:text-content transition-colors p-1 rounded hover:bg-surface-hover"
      title="Sign out"
    >
      <LogOut size={16} />
    </button>
  </div>
</div>

<!-- Body -->
{#if focusSection}
  <button
    onclick={() => onClearFocus?.()}
    class="flex items-center gap-1 px-4 py-2 text-xs text-accent hover:text-accent-hover transition-colors border-b border-border w-full text-left"
  >
    ← All PRs
  </button>
{/if}
<div class="flex-1 overflow-y-auto min-h-0 scrollbar-thin">
  {#if prs.length === 0}
    <div class="flex flex-col items-center justify-center h-full text-content-tertiary gap-2">
      <Inbox size={24} class="text-content-tertiary" />
      <p class="text-sm">No pull requests found</p>
      <p class="text-xs text-content-tertiary">Your PRs will appear here</p>
    </div>
  {:else if visibleSections.length === 0}
    <div class="flex flex-col items-center justify-center h-full text-content-tertiary gap-2">
      <PartyPopper size={24} class="text-content-tertiary" />
      <p class="text-sm">All clear!</p>
    </div>
  {:else}
    <div class="py-1">
      {#each visibleSections as section (section.title)}
        <PRSection {section} expandAll={focusSection === section.title} />
      {/each}
    </div>
  {/if}
</div>

<!-- Footer -->
<div class="flex items-center justify-between px-4 py-2 border-t border-border shrink-0">
  <span class="text-[10px] text-content-tertiary">
    Updated {relativeTime(lastUpdated)}
  </span>
  <div class="flex items-center gap-1">
    <button
      onclick={() => openUrl("https://github.com/pulls")}
      class="flex items-center gap-1 text-[11px] text-content-tertiary hover:text-content-secondary
             transition-colors py-1 px-2 rounded hover:bg-surface-hover"
    >
      <ExternalLink size={12} />
      See all
    </button>
    <button
      onclick={onRefresh}
      disabled={refreshing}
      title="Refresh"
      class="flex h-7 w-7 items-center justify-center text-content-tertiary hover:text-content-secondary
             transition-colors disabled:opacity-50 rounded hover:bg-surface-hover"
    >
      <RefreshCw size={12} class={refreshing ? "animate-spin" : ""} />
    </button>
  </div>
</div>
