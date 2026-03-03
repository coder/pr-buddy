<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
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

<div class="flex flex-col items-center justify-center h-screen bg-[#0d1117] text-white p-6 select-none">
  {#if viewState === "checking"}
    <div class="w-8 h-8 border-2 border-purple-500 border-t-transparent rounded-full animate-spin mb-4"></div>
    <p class="text-sm text-gray-400">Checking for updates...</p>

  {:else if viewState === "up_to_date"}
    <CheckCircle size={40} class="text-green-400 mb-3" />
    <p class="text-lg font-medium mb-1">You're up to date</p>
    <p class="text-sm text-gray-400">Version {currentVersion}</p>

  {:else if viewState === "available"}
    <Download size={40} class="text-purple-400 mb-3" />
    <p class="text-lg font-medium mb-1">Update available</p>
    <p class="text-sm text-gray-400 mb-3">Version {newVersion}</p>
    {#if releaseNotes}
      <div class="w-full max-h-24 overflow-y-auto text-xs text-gray-500 bg-[#161b22] rounded p-2 mb-4 whitespace-pre-wrap">
        {releaseNotes}
      </div>
    {/if}
    <button
      onclick={installUpdate}
      class="px-4 py-2 bg-purple-600 hover:bg-purple-500 rounded text-sm font-medium transition-colors"
    >
      Install &amp; Restart
    </button>

  {:else if viewState === "downloading"}
    <p class="text-sm text-gray-400 mb-3">Downloading update...</p>
    <div class="w-full bg-[#161b22] rounded-full h-2 mb-2">
      <div
        class="bg-purple-500 h-2 rounded-full transition-all duration-200"
        style="width: {downloadProgress}%"
      ></div>
    </div>
    <p class="text-xs text-gray-500">{Math.round(downloadProgress)}%</p>

  {:else if viewState === "error"}
    <AlertCircle size={40} class="text-red-400 mb-3" />
    <p class="text-lg font-medium mb-1">Update failed</p>
    <p class="text-sm text-gray-400 mb-3 text-center max-w-xs">{errorMessage}</p>
    <button
      onclick={checkForUpdate}
      class="px-4 py-2 bg-[#161b22] hover:bg-[#1c2129] rounded text-sm font-medium transition-colors"
    >
      Retry
    </button>
  {/if}
</div>
