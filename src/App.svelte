<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import type { PullRequest, GitHubUser, UserSettings } from "./lib/types";
  import AuthScreen from "./lib/AuthScreen.svelte";
  import PRPanel from "./lib/PRPanel.svelte";
  import SettingsPage from "./lib/SettingsPage.svelte";

  let isAuthed = $state(false);
  let prList = $state<PullRequest[]>([]);
  let userInfo = $state<GitHubUser | null>(null);
  let lastUpdated = $state<Date | null>(null);
  let refreshing = $state(false);
  let checkingAuth = $state(true);
  let view = $state<"panel" | "settings">("panel");
  let settings = $state<UserSettings>({
    notify_checks_failed: true,
    notify_checks_passed: true,
    notify_merged: true,
    notify_removed_from_queue: true,
    hidden_repos: [],
  });

  let unlisten: (() => void) | undefined;
  let unlistenAuth: (() => void) | undefined;

  async function init() {
    // Load settings independently — they're local and should not fail with network errors
    void loadSettings();
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

  async function loadSettings() {
    try {
      const s = await invoke<UserSettings>("get_settings_cmd");
      if (s) settings = s;
    } catch (e) {
      console.error("[app] Failed to load settings:", e);
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

  async function setup() {
    // Await listener registration before init() so we don't miss
    // an auth-cleared event from fast startup token validation.
    unlisten = await listen<PullRequest[]>("prs-updated", (event) => {
      prList = event.payload;
      lastUpdated = new Date();
    });
    unlistenAuth = await listen("auth-cleared", () => {
      isAuthed = false;
      prList = [];
      userInfo = null;
      lastUpdated = null;
    });
    await init();
  }

  onMount(() => {
    void setup();
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
  {:else if view === "settings"}
    <SettingsPage
      prs={prList}
      onBack={() => { view = "panel"; }}
      onSettingsChanged={(s) => { settings = s; }}
    />
  {:else}
    <PRPanel
      prs={prList.filter(pr => !settings.hidden_repos.includes(`${pr.owner}/${pr.repository}`))}
      user={userInfo}
      {lastUpdated}
      {refreshing}
      onRefresh={handleRefresh}
      onLogout={handleLogout}
      onOpenSettings={() => view = "settings"}
    />
  {/if}
</main>
