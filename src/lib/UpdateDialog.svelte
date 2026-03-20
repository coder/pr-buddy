<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount, onDestroy } from "svelte";
  import type { UpdateCheckResult } from "./types";
  import CheckCircle from "@lucide/svelte/icons/check-circle";
  import Download from "@lucide/svelte/icons/download";
  import AlertCircle from "@lucide/svelte/icons/alert-circle";

  type ViewState = "checking" | "up_to_date" | "available" | "downloading" | "error";

  let viewState: ViewState = $state("checking");
  let currentVersion = $state("");
  let newVersion = $state("");
  let releaseNotes = $state("");
  let errorMessage = $state("");
  let downloadProgress = $state(0);
  let unlisten: (() => void) | undefined;

  async function checkForUpdate() {
    viewState = "checking";
    try {
      const result = await invoke<UpdateCheckResult>("check_for_update_cmd");
      currentVersion = result.current_version;
      if (result.update_available) {
        newVersion = result.version ?? "";
        releaseNotes = result.body ?? "";
        viewState = "available";
      } else {
        viewState = "up_to_date";
      }
    } catch (e) {
      errorMessage = String(e);
      viewState = "error";
    }
  }

  async function installUpdate() {
    viewState = "downloading";
    downloadProgress = 0;
    try {
      await invoke("install_update_cmd");
    } catch (e) {
      errorMessage = String(e);
      viewState = "error";
    }
  }

  onMount(() => {
    void (async () => {
      unlisten = await listen<{ chunk_length: number; content_length: number | null }>(
        "update-download-progress",
        (event) => {
          const { chunk_length, content_length } = event.payload;
          if (content_length) {
            downloadProgress = Math.min(
              100,
              downloadProgress + (chunk_length / content_length) * 100
            );
          }
        }
      );
      await checkForUpdate();
    })();
  });

  onDestroy(() => {
    unlisten?.();
  });
</script>

<div class="flex min-h-0 flex-1 select-none items-center justify-center bg-surface p-6 text-content">
  <div class="w-full max-w-sm">
    {#if viewState === "checking"}
      <div class="rounded-xl bg-surface-secondary p-5 text-center">
        <div class="mb-4 inline-flex h-8 w-8 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
        <p class="mb-1 text-lg font-medium">Checking for updates</p>
        <p class="text-sm text-content-secondary">Looking for the latest PR Buddy release…</p>
      </div>

    {:else if viewState === "up_to_date"}
      <div class="rounded-xl bg-surface-secondary p-5 text-center">
        <CheckCircle size={32} class="mx-auto mb-3 text-emerald-500" />
        <p class="mb-1 text-lg font-medium">You're up to date</p>
        <p class="text-sm text-content-secondary">Version {currentVersion}</p>
      </div>
      <div class="mt-3 overflow-hidden rounded-xl bg-surface-secondary">
        <button
          onclick={() => getCurrentWindow().close()}
          class="w-full px-3 py-2.5 text-center text-sm font-medium text-content-secondary transition-colors hover:bg-surface-hover"
        >
          Close
        </button>
      </div>

    {:else if viewState === "available"}
      <div class="rounded-xl bg-surface-secondary p-5 text-center">
        <Download size={32} class="mx-auto mb-3 text-accent" />
        <p class="mb-1 text-lg font-medium">Update available</p>
        <p class="mb-3 text-sm text-content-secondary">Version {newVersion}</p>
        {#if releaseNotes}
          <div class="max-h-24 w-full overflow-y-auto whitespace-pre-wrap rounded-lg bg-surface p-2 text-[11px] text-content-tertiary">
            {releaseNotes}
          </div>
        {/if}
      </div>
      <div class="mt-3 overflow-hidden rounded-xl bg-surface-secondary">
        <button
          onclick={installUpdate}
          class="w-full px-3 py-2.5 text-center text-sm font-medium text-accent transition-colors hover:bg-surface-hover"
        >
          Install &amp; Restart
        </button>
      </div>

    {:else if viewState === "downloading"}
      <div class="rounded-xl bg-surface-secondary p-5 text-center">
        <Download size={32} class="mx-auto mb-3 text-accent" />
        <p class="mb-1 text-lg font-medium">Downloading update</p>
        <p class="mb-4 text-sm text-content-secondary">Please keep PR Buddy open while the update downloads.</p>
        <div class="mb-2 w-full rounded-full bg-surface h-1.5">
          <div
            class="h-1.5 rounded-full bg-accent transition-all duration-200"
            style="width: {downloadProgress}%"
          ></div>
        </div>
        <p class="text-xs text-content-tertiary">{Math.round(downloadProgress)}%</p>
      </div>

    {:else if viewState === "error"}
      <div class="rounded-xl bg-surface-secondary p-5 text-center">
        <AlertCircle size={32} class="mx-auto mb-3 text-red-400" />
        <p class="mb-1 text-lg font-medium">Update failed</p>
        <p class="text-sm text-content-secondary">{errorMessage}</p>
      </div>
      <div class="mt-3 overflow-hidden rounded-xl bg-surface-secondary">
        <button
          onclick={checkForUpdate}
          class="w-full px-3 py-2.5 text-center text-sm font-medium text-content-secondary transition-colors hover:bg-surface-hover"
        >
          Retry
        </button>
      </div>
    {/if}
  </div>
</div>
