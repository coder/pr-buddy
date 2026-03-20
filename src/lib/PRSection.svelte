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
  let visiblePrs = $derived(section.prs.slice(0, visibleCount));
  let hasMore = $derived(visibleCount < section.prs.length);
  let remaining = $derived(Math.max(section.prs.length - visibleCount, 0));
  let effectiveCollapsed = $derived(expandAll ? false : collapsed);
  let effectiveVisiblePrs = $derived(expandAll ? section.prs : visiblePrs);
  let effectiveHasMore = $derived(expandAll ? false : hasMore);
  let effectiveRemaining = $derived(expandAll ? 0 : remaining);

  function destinationUrlFor(pr: (typeof section.prs)[number]): string {
    return section.title === "Checks Failing" ? `${pr.url}/checks` : pr.url;
  }

  function toggle() {
    collapsed = !collapsed;
  }

  function showMore() {
    visibleCount += 5;
  }
</script>

<div>
  <button onclick={toggle} class="flex w-full items-center justify-between px-1 pb-1 pt-0.5 text-left">
    <span class="text-[11px] font-medium text-content-secondary">
      {section.title}
      <span class="ml-1 text-content-tertiary">{section.prs.length}</span>
    </span>
    <svg
      class={`h-3 w-3 text-content-tertiary transition-transform ${effectiveCollapsed ? "-rotate-90" : ""}`}
      fill="none"
      stroke="currentColor"
      viewBox="0 0 24 24"
      stroke-width="2"
      aria-hidden="true"
    >
      <path stroke-linecap="round" stroke-linejoin="round" d="M19 9l-7 7-7-7" />
    </svg>
  </button>

  {#if !effectiveCollapsed}
    <div class="overflow-hidden rounded-xl bg-surface-secondary">
      {#each effectiveVisiblePrs as pr, i (pr.id)}
        {#if i > 0}
          <div class="mx-3 border-t border-border/30"></div>
        {/if}
        <PRCard {pr} destinationUrl={destinationUrlFor(pr)} />
      {/each}

      {#if effectiveHasMore}
        <div class="mx-3 border-t border-border/30"></div>
        <button
          onclick={showMore}
          class="w-full py-1.5 text-center text-[10px] text-accent hover:text-accent-hover"
        >
          Show {effectiveRemaining} more
        </button>
      {/if}
    </div>
  {/if}
</div>
