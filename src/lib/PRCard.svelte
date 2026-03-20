<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import MessageSquare from "@lucide/svelte/icons/message-square";
  import type { PullRequest } from "./types";
  import StatusBadge from "./StatusBadge.svelte";

  interface Props {
    pr: PullRequest;
    destinationUrl?: string;
  }
  let { pr, destinationUrl = pr.url }: Props = $props();

  let repoDisplay = $derived(`${pr.owner}/${pr.repository}`);

  function timeAgo(iso: string): string {
    const diff = Math.floor((Date.now() - new Date(iso).getTime()) / 1000);
    if (diff < 60) return `${diff}s`;
    if (diff < 3600) return `${Math.floor(diff / 60)}m`;
    if (diff < 86400) return `${Math.floor(diff / 3600)}h`;
    return `${Math.floor(diff / 86400)}d`;
  }

  let age = $derived(timeAgo(pr.created_at));

  async function openPr() {
    await openUrl(destinationUrl);
  }
</script>

<button
  onclick={openPr}
  class="group flex w-full cursor-pointer items-center gap-2.5 px-3 py-2 text-left transition-colors hover:bg-surface-hover"
>
  {#if pr.author_avatar_url}
    <img src={pr.author_avatar_url} alt={pr.author_login} class="h-5 w-5 shrink-0 rounded-full" />
  {/if}

  <StatusBadge {pr} />

  <div class="min-w-0 flex-1">
    <p class="truncate text-[12px] leading-tight text-content group-hover:text-accent">
      {pr.title}
    </p>
    <div class="mt-0.5 flex items-center gap-1.5 text-[10px]">
      <span class="text-content-tertiary">{repoDisplay}</span>
      <span class="text-content-tertiary">·</span>
      <span class="text-content-tertiary">#{pr.number}</span>
      {#if pr.additions > 0 || pr.deletions > 0}
        <span class="text-content-tertiary">·</span>
        <span class="text-green-600/80">+{pr.additions}</span>
        <span class="text-red-500/80">-{pr.deletions}</span>
      {/if}
    </div>
  </div>

  <div class="flex shrink-0 items-center gap-1.5 text-content-tertiary">
    {#if pr.comment_count > 0}
      <span class="flex items-center gap-0.5 text-[10px]">
        <MessageSquare size={10} />
        {pr.comment_count}
      </span>
    {/if}
    <span class="text-[10px]">{age}</span>
  </div>
</button>
