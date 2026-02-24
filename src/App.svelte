<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import AuthScreen from "./lib/AuthScreen.svelte";
  import PRPanel from "./lib/PRPanel.svelte";
  import type { PullRequest, GitHubUser } from "./lib/types";

  let authenticated = $state(false);
  let loading = $state(true);
  let prList = $state<PullRequest[]>([]);
  let userInfo = $state<GitHubUser | null>(null);
  let lastUpdated = $state<Date | null>(null);
  let refreshing = $state(false);

  onMount(async () => {
    try {
      authenticated = await invoke<boolean>("is_authenticated_cmd");
      if (authenticated) {
        await loadData();
      }
    } catch {
      authenticated = false;
    } finally {
      loading = false;
    }

    const unlisten = await listen<PullRequest[]>("prs-updated", (event) => {
      prList = event.payload;
      lastUpdated = new Date();
    });

    return () => { unlisten(); };
  });

  async function loadData() {
    const [prs, user] = await Promise.all([
      invoke<PullRequest[]>("get_pull_requests_cmd"),
      invoke<GitHubUser | null>("get_user_info_cmd"),
    ]);
    prList = prs;
    userInfo = user ?? null;
    lastUpdated = new Date();
  }

  async function handleRefresh() {
    refreshing = true;
    try {
      prList = await invoke<PullRequest[]>("refresh_prs_cmd");
      lastUpdated = new Date();
    } finally {
      refreshing = false;
    }
  }

  async function handleAuthSuccess() {
    authenticated = true;
    await loadData();
  }

  async function handleLogout() {
    await invoke("logout_cmd");
    authenticated = false;
    prList = [];
    userInfo = null;
    lastUpdated = null;
  }
</script>

<main class="h-screen bg-[#12121a] text-white flex flex-col overflow-hidden">
  {#if loading}
    <div class="flex items-center justify-center h-full">
      <div class="flex flex-col items-center gap-3">
        <div class="w-6 h-6 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
        <p class="text-gray-500 text-sm">Loading…</p>
      </div>
    </div>
  {:else if !authenticated}
    <AuthScreen onSuccess={handleAuthSuccess} />
  {:else}
    <PRPanel
      prs={prList}
      user={userInfo}
      {lastUpdated}
      {refreshing}
      onRefresh={handleRefresh}
      onLogout={handleLogout}
    />
  {/if}
</main>
