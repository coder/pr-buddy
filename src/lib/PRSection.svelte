<script lang="ts">
  import type { PrSection } from "./types";
  import PRCard from "./PRCard.svelte";

  interface Props {
    section: PrSection;
    expandAll?: boolean;
  }
  let { section, expandAll = false }: Props = $props();

  const getInitialCollapsed = () => section.defaultCollapsed ?? false;
  let collapsed = $state(getInitialCollapsed());
  let visibleCount = $state(5);
  let SectionIcon = $derived(section.icon);
  let visiblePrs = $derived(section.prs.slice(0, visibleCount));
  let hasMore = $derived(visibleCount < section.prs.length);
  let effectiveCollapsed = $derived(expandAll ? false : collapsed);
  let effectiveVisiblePrs = $derived(expandAll ? section.prs : visiblePrs);
  let effectiveHasMore = $derived(expandAll ? false : hasMore);

  function destinationUrlFor(pr: (typeof section.prs)[number]): string {
    return section.title === "Checks Failing" ? `${pr.url}/checks` : pr.url;
  }
</script>

<div class="border-b border-border last:border-b-0">
  <button
    onclick={() => { collapsed = !collapsed; }}
    class="w-full flex items-center justify-between px-4 py-2 hover:bg-surface-hover
           transition-colors text-left"
  >
    <div class="flex items-center gap-2">
      <SectionIcon size={14} class="text-content-secondary" />
      <span class="text-xs font-medium text-content-secondary">{section.title}</span>
      <span class="text-[10px] text-content-tertiary bg-surface-secondary px-1.5 py-0.5 rounded-full">
        {section.prs.length}
      </span>
    </div>
    <svg
      class="w-3 h-3 text-content-tertiary transition-transform {effectiveCollapsed ? '-rotate-90' : ''}"
      fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="2"
    >
      <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if !effectiveCollapsed}
    <div class="pb-1">
      {#each effectiveVisiblePrs as pr (pr.id)}
        <PRCard {pr} destinationUrl={destinationUrlFor(pr)} />
      {/each}

      {#if effectiveHasMore}
        <button
          onclick={() => { visibleCount += 5; }}
          class="w-full text-[11px] text-content-tertiary hover:text-content-secondary py-1.5 px-4
                 hover:bg-surface-hover transition-colors text-left"
        >
          Show more ({section.prs.length - visibleCount} remaining)
        </button>
      {/if}
    </div>
  {/if}
</div>
