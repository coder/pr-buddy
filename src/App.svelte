<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type { PullRequest, GitHubUser } from "./lib/types";
  import AuthScreen from "./lib/AuthScreen.svelte";
  import PRPanel from "./lib/PRPanel.svelte";

  let isAuthed = $state(false);
  let prList = $state<PullRequest[]>([]);
  let userInfo = $state<GitHubUser | null>(null);
  let lastUpdated = $state<Date | null>(null);
  let refreshing = $state(false);
  let checkingAuth = $state(true);

  let unlisten: (() => void) | undefined;
  let unlistenAuth: (() => void) | undefined;

  async function init() {
    try {
      isAuthed = await invoke<boolean>("is_authenticated_cmd");
      if (isAuthed) {
        await loadData();
      }
    } catch (e) {
      console.error("[app] Auth check failed:", e);
    } finally {
      checkingAuth = false;
    }
  }

  async function loadData() {
    try {
      const [prs, user] = await Promise.all([
        invoke<PullRequest[]>("get_pull_requests_cmd"),
        invoke<GitHubUser>("get_user_info_cmd"),
      ]);
      prList = prs;
      userInfo = user;
      lastUpdated = new Date();
    } catch (e) {
      console.error("[app] Failed to load data:", e);
    }
  }

  async function handleRefresh() {
    refreshing = true;
    try {
      prList = await invoke<PullRequest[]>("refresh_prs_cmd");
      lastUpdated = new Date();
    } catch (e) {
      console.error("[app] Refresh failed:", e);
    } finally {
      refreshing = false;
    }
  }

  async function handleLogout() {
    try {
      await invoke("logout_cmd");
      isAuthed = false;
      prList = [];
      userInfo = null;
      lastUpdated = null;
    } catch (e) {
      console.error("[app] Logout failed:", e);
    }
  }

  async function handleAuthSuccess() {
    isAuthed = true;
    await loadData();
  }

  onMount(() => {
    // Register event listeners before init() so we don't miss
    // an auth-cleared event from fast startup token validation.
    listen<PullRequest[]>("prs-updated", (event) => {
      prList = event.payload;
      lastUpdated = new Date();
    }).then((fn) => {
      unlisten = fn;
    });
    listen("auth-cleared", () => {
      isAuthed = false;
      prList = [];
      userInfo = null;
      lastUpdated = null;
    }).then((fn) => {
      unlistenAuth = fn;
    });
    void init();
  });

  onDestroy(() => {
    unlisten?.();
    unlistenAuth?.();
  });
</script>

<main class="flex flex-col h-screen bg-[#0d1117] text-white overflow-hidden select-none">
  {#if checkingAuth}
    <div class="flex items-center justify-center h-full">
      <div class="w-6 h-6 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else if !isAuthed}
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
