<script lang="ts">
  import { openUrl } from "@tauri-apps/plugin-opener";
  import type { PullRequest } from "./types";
  import StatusBadge from "./StatusBadge.svelte";

  interface Props {
    pr: PullRequest;
  }
  let { pr }: Props = $props();

  let repoDisplay = $derived(`${pr.owner}/${pr.repository}`);

  async function openPr() {
    await openUrl(pr.url);
  }
</script>

<button
  onclick={openPr}
  class="w-full flex items-center gap-2.5 px-4 py-2 hover:bg-[#1e1e2e] transition-colors
         text-left group cursor-pointer"
>
  <StatusBadge {pr} />

  <div class="flex-1 min-w-0">
    <p class="text-[13px] text-gray-200 truncate leading-tight group-hover:text-white transition-colors">
      {pr.title}
    </p>
    <div class="flex items-center gap-1.5 mt-0.5">
      <span class="text-[11px] text-gray-600">{repoDisplay}</span>
      <span class="text-[11px] text-gray-700">·</span>
      <span class="text-[11px] text-gray-600">#{pr.number}</span>
      {#if pr.additions > 0 || pr.deletions > 0}
        <span class="text-[11px] text-gray-700">·</span>
        <span class="text-[10px] text-green-600">+{pr.additions}</span>
        <span class="text-[10px] text-red-500">-{pr.deletions}</span>
      {/if}
    </div>
  </div>
</button>
