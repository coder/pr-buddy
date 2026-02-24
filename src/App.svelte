<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let authenticated = $state(false);
  let loading = $state(true);

  onMount(async () => {
    try {
      authenticated = await invoke<boolean>("is_authenticated_cmd");
    } catch {
      authenticated = false;
    } finally {
      loading = false;
    }
  });
</script>

<main class="h-screen bg-gray-900 text-white p-4">
  {#if loading}
    <div class="flex items-center justify-center h-full">
      <p class="text-gray-400">Loading...</p>
    </div>
  {:else if authenticated}
    <div class="space-y-4">
      <h1 class="text-lg font-semibold">PR Buddy</h1>
      <p class="text-gray-400">Your pull requests will appear here.</p>
    </div>
  {:else}
    <div class="flex flex-col items-center justify-center h-full space-y-4">
      <h1 class="text-xl font-bold">PR Buddy</h1>
      <p class="text-gray-400">Sign in with GitHub to get started.</p>
    </div>
  {/if}
</main>
