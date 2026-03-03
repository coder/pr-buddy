<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Bell from "@lucide/svelte/icons/bell";
  import GitBranch from "@lucide/svelte/icons/git-branch";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { PullRequest, UserSettings } from "./types";

  interface Props {
    prs: PullRequest[];
    onBack: () => void;
    onSettingsChanged: (s: UserSettings) => void;
  }
  let { prs, onBack, onSettingsChanged }: Props = $props();

  let settings = $state<UserSettings>({
    notify_checks_failed: true,
    notify_checks_passed: true,
    notify_merged: true,
    notify_removed_from_queue: true,
    hidden_repos: [],
  });

  let loaded = $state(false);

  // Serialize saves so rapid toggles don't race each other
  let saveChain = Promise.resolve();

  onMount(() => {
    void loadSettings();
  });

  async function loadSettings() {
    try {
      const s = await invoke<UserSettings>("get_settings_cmd");
      if (s) settings = s;
    } catch (e) {
      console.error("[settings] Failed to load settings:", e);
    } finally {
      loaded = true;
    }
  }

  function save() {
    // Notify parent immediately so panel filter stays in sync
    onSettingsChanged(structuredClone(settings));
    saveChain = saveChain
      .then(async () => { await invoke("save_settings_cmd", { settings }); })
      .catch((e: unknown) => console.error("[settings] Failed to save settings:", e));
  }

  // Derive unique repos from current PRs + any hidden repos not in current PRs
  let allRepos = $derived.by(() => {
    const fromPrs = prs.map((pr) => `${pr.owner}/${pr.repository}`);
    const combined = new Set([...fromPrs, ...settings.hidden_repos]);
    return [...combined].sort();
  });

  function toggleNotification(key: keyof UserSettings) {
    (settings as any)[key] = !(settings as any)[key];
    void save();
  }

  function toggleRepo(repo: string) {
    const idx = settings.hidden_repos.indexOf(repo);
    if (idx >= 0) {
      settings.hidden_repos = settings.hidden_repos.filter((r) => r !== repo);
    } else {
      settings.hidden_repos = [...settings.hidden_repos, repo];
    }
    void save();
  }

  function showAll() {
    settings.hidden_repos = [];
    void save();
  }

  function hideAll() {
    settings.hidden_repos = [...allRepos];
    void save();
  }

  let allHidden = $derived(
    allRepos.length > 0 && settings.hidden_repos.length === allRepos.length,
  );

  const notificationItems: { key: keyof UserSettings; label: string; emoji: string }[] = [
    { key: "notify_checks_failed", label: "Checks failed", emoji: "❌" },
    { key: "notify_checks_passed", label: "Checks passed", emoji: "✅" },
    { key: "notify_merged", label: "PR merged", emoji: "🎉" },
    { key: "notify_removed_from_queue", label: "Removed from merge queue", emoji: "🚫" },
  ];
</script>

<!-- Header -->
<div class="flex items-center gap-2 px-4 py-3 border-b border-gray-800 shrink-0">
  <button
    onclick={onBack}
    class="text-gray-400 hover:text-white transition-colors p-1 rounded hover:bg-[#1e1e2e]"
    title="Back"
  >
    <ArrowLeft size={16} />
  </button>
  <h1 class="text-sm font-semibold text-white">Settings</h1>
</div>

<!-- Body -->
<div class="flex-1 overflow-y-auto min-h-0 scrollbar-thin px-4 py-3 space-y-5">
  {#if !loaded}
    <div class="flex items-center justify-center py-8">
      <div class="w-5 h-5 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
    </div>
  {:else}
    <!-- Notifications -->
    <section>
      <div class="flex items-center gap-1.5 mb-2">
        <Bell size={13} class="text-gray-400" />
        <h2 class="text-xs font-semibold text-gray-400 uppercase tracking-wide">Notifications</h2>
      </div>
      <div class="space-y-1">
        {#each notificationItems as item (item.key)}
          <button
            onclick={() => toggleNotification(item.key)}
            class="flex items-center justify-between w-full px-3 py-2 rounded-lg
                   hover:bg-[#1e1e2e] transition-colors text-left"
          >
            <span class="flex items-center gap-2 text-sm text-gray-300">
              <span class="text-xs">{item.emoji}</span>
              {item.label}
            </span>
            <div
              class="w-8 h-[18px] rounded-full transition-colors relative
                     {(settings as any)[item.key] ? 'bg-purple-600' : 'bg-gray-700'}"
            >
              <div
                class="absolute top-[2px] w-[14px] h-[14px] rounded-full bg-white transition-transform
                       {(settings as any)[item.key] ? 'translate-x-[16px]' : 'translate-x-[2px]'}"
              ></div>
            </div>
          </button>
        {/each}
      </div>
    </section>

    <!-- Repositories -->
    <section>
      <div class="flex items-center justify-between mb-2">
        <div class="flex items-center gap-1.5">
          <GitBranch size={13} class="text-gray-400" />
          <h2 class="text-xs font-semibold text-gray-400 uppercase tracking-wide">Repositories</h2>
        </div>
        {#if allRepos.length > 0}
          <button
            onclick={() => allHidden ? showAll() : hideAll()}
            class="text-[10px] text-purple-400 hover:text-purple-300 transition-colors"
          >
            {allHidden ? "Show all" : "Hide all"}
          </button>
        {/if}
      </div>
      {#if allRepos.length === 0}
        <p class="text-xs text-gray-600 px-3 py-2">No repositories yet</p>
      {:else}
        <div class="space-y-0.5">
          {#each allRepos as repo (repo)}
            <button
              onclick={() => toggleRepo(repo)}
              class="flex items-center gap-2.5 w-full px-3 py-1.5 rounded-lg
                     hover:bg-[#1e1e2e] transition-colors text-left"
            >
              <div
                class="w-3.5 h-3.5 rounded border flex items-center justify-center shrink-0
                       {settings.hidden_repos.includes(repo)
                         ? 'border-gray-600 bg-transparent'
                         : 'border-purple-600 bg-purple-600'}"
              >
                {#if !settings.hidden_repos.includes(repo)}
                  <svg class="w-2.5 h-2.5 text-white" viewBox="0 0 12 12" fill="none">
                    <path d="M2 6l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                {/if}
              </div>
              <span class="text-sm text-gray-300 truncate">{repo}</span>
            </button>
          {/each}
        </div>
      {/if}
    </section>
  {/if}
</div>
