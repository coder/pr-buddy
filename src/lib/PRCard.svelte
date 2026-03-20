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
  class="w-full flex items-center gap-2.5 px-4 py-1.5 hover:bg-surface-hover transition-colors
         text-left group cursor-pointer"
>
  <!-- Author avatar -->
  {#if pr.author_avatar_url}
    <img src={pr.author_avatar_url} alt={pr.author_login} class="w-6 h-6 rounded-full shrink-0" />
  {/if}

  <StatusBadge {pr} />

  <div class="flex-1 min-w-0">
    <p class="text-[12px] text-content truncate leading-tight group-hover:text-accent-blue">
      {pr.title}
    </p>
    <div class="flex items-center gap-1.5 mt-0.5 text-[10px]">
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

  <!-- Right-aligned metadata: comment count + age -->
  <div class="shrink-0 flex items-center gap-1.5 text-content-tertiary">
    {#if pr.comment_count > 0}
      <span class="flex items-center gap-0.5 text-[10px]">
        <MessageSquare size={10} />
        {pr.comment_count}
      </span>
    {/if}
    <span class="text-[10px]">{age}</span>
  </div>
</button>
