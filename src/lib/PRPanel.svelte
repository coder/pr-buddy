<script lang="ts">
  import Bell from "@lucide/svelte/icons/bell";
  import Inbox from "@lucide/svelte/icons/inbox";
  import PartyPopper from "@lucide/svelte/icons/party-popper";
  import RefreshCw from "@lucide/svelte/icons/refresh-cw";
  import ExternalLink from "@lucide/svelte/icons/external-link";
  import LogOut from "@lucide/svelte/icons/log-out";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
  import type { PullRequest, GitHubUser } from "./types";
  import { groupPrs } from "./stores";
  import PRSection from "./PRSection.svelte";
  import { onMount } from "svelte";

  interface Props {
    prs: PullRequest[];
    user: GitHubUser | null;
    lastUpdated: Date | null;
    refreshing: boolean;
    onRefresh: () => void;
    onLogout: () => void;
  }
  let { prs, user, lastUpdated, refreshing, onRefresh, onLogout }: Props = $props();

  let now = $state(new Date());
  $effect(() => {
    const id = setInterval(() => { now = new Date(); }, 1000);
    return () => clearInterval(id);
  });

  let autostart = $state(false);
  let toggling = $state(false);
  let toggled = false;
  onMount(() => {
    void isEnabled().then((v) => {
      if (!toggled) autostart = v;
    });
  });

  async function toggleAutostart() {
    if (toggling) return;
    toggling = true;
    toggled = true;
    try {
      if (autostart) {
        await disable();
      } else {
        await enable();
      }
      autostart = await isEnabled();
    } catch (e) {
      console.error("Failed to toggle autostart:", e);
    } finally {
      toggling = false;
    }
  }

  let sections = $derived(groupPrs(prs));
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
<div class="flex items-center justify-between px-4 py-3 border-b border-gray-800 shrink-0">
  <div class="flex items-center gap-2">
    <Bell size={16} class="text-gray-400" />
    <h1 class="text-sm font-semibold text-white">PR Buddy</h1>
    {#if totalPrs > 0}
      <span class="text-[10px] bg-purple-600 text-white px-1.5 py-0.5 rounded-full font-medium">
        {totalPrs}
      </span>
    {/if}
  </div>
  <div class="flex items-center gap-2">
    {#if user}
      <img
        src={user.avatar_url}
        alt={user.login}
        class="w-6 h-6 rounded-full ring-1 ring-gray-700"
      />
    {/if}
    <button
      onclick={onLogout}
      class="text-gray-500 hover:text-gray-300 transition-colors p-1 rounded hover:bg-[#1e1e2e]"
      title="Sign out"
    >
      <LogOut size={16} />
    </button>
  </div>
</div>

<!-- Body -->
<div class="flex-1 overflow-y-auto min-h-0 scrollbar-thin">
  {#if prs.length === 0}
    <div class="flex flex-col items-center justify-center h-full text-gray-500 gap-2">
      <Inbox size={24} class="text-gray-500" />
      <p class="text-sm">No pull requests found</p>
      <p class="text-xs text-gray-600">Your PRs will appear here</p>
    </div>
  {:else if sections.length === 0}
    <div class="flex flex-col items-center justify-center h-full text-gray-500 gap-2">
      <PartyPopper size={24} class="text-gray-500" />
      <p class="text-sm">All clear!</p>
    </div>
  {:else}
    <div class="py-1">
      {#each sections as section (section.title)}
        <PRSection {section} />
      {/each}
    </div>
  {/if}
</div>

<!-- Footer -->
<div class="border-t border-gray-800 shrink-0">
  <div class="flex items-center justify-between px-4 py-2">
    <span class="text-[11px] text-gray-600">
      Updated {relativeTime(lastUpdated)}
    </span>
    <div class="flex items-center gap-1">
      <button
        onclick={() => openUrl("https://github.com/pulls")}
        class="flex items-center gap-1 text-[11px] text-gray-500 hover:text-gray-300
               transition-colors py-1 px-2 rounded hover:bg-[#1e1e2e]"
      >
        <ExternalLink size={12} />
        See all
      </button>
      <button
        onclick={onRefresh}
        disabled={refreshing}
        class="flex items-center gap-1 text-[11px] text-gray-500 hover:text-gray-300
               transition-colors disabled:opacity-50 py-1 px-2 rounded hover:bg-[#1e1e2e]"
      >
        <RefreshCw size={12} class={refreshing ? "animate-spin" : ""} />
        Refresh
      </button>
    </div>
  </div>
  <div class="flex items-center justify-between px-4 py-1.5 border-t border-gray-800/50">
    <label class="flex items-center gap-2 cursor-pointer select-none">
      <button
        role="switch"
        aria-checked={autostart}
        aria-label="Launch at login"
        onclick={toggleAutostart}
        class="relative inline-flex h-4 w-7 items-center rounded-full transition-colors
               {autostart ? 'bg-purple-600' : 'bg-gray-700'}"
      >
        <span
          class="inline-block h-3 w-3 rounded-full bg-white transition-transform
                 {autostart ? 'translate-x-3.5' : 'translate-x-0.5'}"
        ></span>
      </button>
      <span class="text-[11px] text-gray-500">Launch at login</span>
    </label>
  </div>
</div>
