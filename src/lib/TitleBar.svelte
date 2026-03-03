<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Minus from "@lucide/svelte/icons/minus";
  import X from "@lucide/svelte/icons/x";
  import Bell from "@lucide/svelte/icons/bell";
</script>

<div
  data-tauri-drag-region
  class="flex items-center justify-between h-9 px-3
         bg-surface border-b border-border shrink-0 select-none"
>
  <div class="flex items-center gap-2 pointer-events-none">
    <Bell size={16} class="text-content-secondary" />
    <span class="text-xs font-semibold text-content">PR Buddy</span>
  </div>
  <div class="flex items-center gap-0.5">
    <button
      onclick={() => getCurrentWindow().minimize()}
      class="p-1 rounded text-content-tertiary hover:text-content
             hover:bg-surface-hover transition-colors"
    >
      <Minus size={14} />
    </button>
    <button
      onclick={() => getCurrentWindow().hide()}
      class="p-1 rounded text-content-tertiary hover:text-content
             hover:bg-surface-hover transition-colors"
    >
      <X size={14} />
    </button>
  </div>
</div>
