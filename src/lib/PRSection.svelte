<script lang="ts">
  import type { PrSection } from "./types";
  import PRCard from "./PRCard.svelte";

  interface Props {
    section: PrSection;
  }
  let { section }: Props = $props();

  let collapsed = $state(false);
  let visibleCount = $state(5);
  let SectionIcon = $derived(section.icon);
  let visiblePrs = $derived(section.prs.slice(0, visibleCount));
  let hasMore = $derived(visibleCount < section.prs.length);
</script>

<div class="border-b border-gray-800/50 last:border-b-0">
  <button
    onclick={() => { collapsed = !collapsed; }}
    class="w-full flex items-center justify-between px-4 py-2 hover:bg-[#1e1e2e]/50
           transition-colors text-left"
  >
    <div class="flex items-center gap-2">
      <SectionIcon size={14} class="text-gray-400" />
      <span class="text-xs font-medium text-gray-400">{section.title}</span>
      <span class="text-[10px] text-gray-600 bg-gray-800 px-1.5 py-0.5 rounded-full">
        {section.prs.length}
      </span>
    </div>
    <svg
      class="w-3 h-3 text-gray-600 transition-transform {collapsed ? '-rotate-90' : ''}"
      fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"
    >
      <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if !collapsed}
    <div class="pb-1">
      {#each visiblePrs as pr (pr.id)}
        <PRCard {pr} />
      {/each}

      {#if hasMore}
        <button
          onclick={() => { visibleCount += 5; }}
          class="w-full text-[11px] text-gray-500 hover:text-gray-300 py-1.5 px-4
                 hover:bg-[#1e1e2e]/50 transition-colors text-left"
        >
          Show more ({section.prs.length - visibleCount} remaining)
        </button>
      {/if}
    </div>
  {/if}
</div>
