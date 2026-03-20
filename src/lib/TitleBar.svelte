<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import Minus from "@lucide/svelte/icons/minus";
  import X from "@lucide/svelte/icons/x";
</script>

<div
  data-tauri-drag-region
  class="flex items-center justify-between h-8 px-2.5
         bg-surface border-b border-border shrink-0 select-none"
>
  <div class="pointer-events-none">
    <span class="text-[11px] font-medium text-content-secondary">PR Buddy</span>
  </div>
  <div class="flex items-center gap-0.5">
    <button
      onclick={() => getCurrentWindow().minimize()}
      class="flex h-6 w-6 items-center justify-center rounded-md text-content-tertiary
             hover:bg-surface-hover hover:text-content transition-colors"
    >
      <Minus size={14} />
    </button>
    <button
      onclick={() => {
        const win = getCurrentWindow();
        if (win.label === "main") {
          win.hide();
        } else {
          win.close();
        }
      }}
      class="flex h-6 w-6 items-center justify-center rounded-md text-content-tertiary
             hover:bg-surface-hover hover:text-content transition-colors"
    >
      <X size={14} />
    </button>
  </div>
</div>
