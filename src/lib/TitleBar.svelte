<script lang="ts">
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import X from "@lucide/svelte/icons/x";

  function dismiss() {
    const win = getCurrentWindow();
    if (win.label === "main") {
      win.hide();
    } else {
      win.close();
    }
  }
</script>

<div data-tauri-drag-region class="flex items-center justify-end px-2 pt-1.5 pb-0 shrink-0">
  <button
    onclick={dismiss}
    class="rounded-full p-0.5 text-content-tertiary transition-colors hover:bg-surface-hover hover:text-content-secondary"
    title="Close"
  >
    <X size={12} />
  </button>
</div>
