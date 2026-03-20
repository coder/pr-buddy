<script lang="ts">
  import ArrowLeft from "@lucide/svelte/icons/arrow-left";
  import Moon from "@lucide/svelte/icons/moon";
  import Monitor from "@lucide/svelte/icons/monitor";
  import Sun from "@lucide/svelte/icons/sun";
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import type { ThemePreference } from "./theme.svelte.ts";
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
  let currentTheme = $state<ThemePreference>("system");
  let setThemePreference: (t: ThemePreference) => void = () => {};
  let launchAtLoginEnabled = $state(false);
  let autoStartLoaded = $state(false);
  let autoStartToggleCount = 0; // generation counter to discard stale reads/rollbacks

  // Serialize saves so rapid toggles don't race each other
  let saveChain = Promise.resolve();
  let autoStartChain = Promise.resolve();

  onMount(() => {
    void loadSettings();
    void loadAutostartSetting();
    void initTheme();
  });

  async function loadAutostartSetting() {
    const gen = autoStartToggleCount;
    try {
      const enabled = await invoke<boolean>("is_autostart_enabled_cmd");
      // Only apply if user hasn't toggled while we were loading
      if (autoStartToggleCount === gen) {
        launchAtLoginEnabled = Boolean(enabled);
      }
    } catch (e) {
      console.error("[settings] Failed to load autostart setting:", e);
    } finally {
      autoStartLoaded = true;
    }
  }

  async function initTheme() {
    if (typeof window !== "undefined" && typeof window.matchMedia !== "function") {
      window.matchMedia = (() => ({
        matches: false,
        media: "",
        onchange: null,
        addListener: () => {},
        removeListener: () => {},
        addEventListener: () => {},
        removeEventListener: () => {},
        dispatchEvent: () => false,
      })) as typeof window.matchMedia;
    }

    const { getTheme, setTheme } = await import("./theme.svelte");
    setThemePreference = setTheme;
    currentTheme = getTheme();
  }

  function selectTheme(theme: ThemePreference) {
    setThemePreference(theme);
    currentTheme = theme;
  }
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
    onSettingsChanged($state.snapshot(settings));
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

  function toggleLaunchAtLogin() {
    const enabled = !launchAtLoginEnabled;
    launchAtLoginEnabled = enabled;
    const gen = ++autoStartToggleCount;
    autoStartChain = autoStartChain
      .then(async () => { await invoke("set_autostart_cmd", { enabled }); })
      .catch((e: unknown) => {
        console.error("[settings] Failed to set autostart setting:", e);
        // Only rollback if no newer toggle has happened since
        if (autoStartToggleCount === gen) {
          launchAtLoginEnabled = !enabled;
        }
      });
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
<div class="flex items-center gap-2 bg-transparent px-3 py-2 shrink-0">
  <button
    onclick={onBack}
    class="rounded-lg p-1 text-content-secondary transition-colors hover:bg-surface-hover hover:text-content"
    title="Back"
  >
    <ArrowLeft size={16} />
  </button>
  <h1 class="text-sm font-semibold text-content">Settings</h1>
</div>

<!-- Body -->
<div class="flex-1 overflow-y-auto min-h-0 scrollbar-thin px-3 py-2 space-y-4">
  {#if !loaded}
    <div class="flex items-center justify-center py-8">
      <div class="h-5 w-5 animate-spin rounded-full border-2 border-accent border-t-transparent"></div>
    </div>
  {:else}
    <!-- General -->
    <section>
      <h2 class="px-1 pb-1 text-[11px] font-medium text-content-secondary">General</h2>
      <div class="overflow-hidden rounded-xl bg-surface-secondary">
        <button
          onclick={toggleLaunchAtLogin}
          disabled={!autoStartLoaded}
          class="flex w-full items-center justify-between px-3 py-2.5 text-left transition-colors hover:bg-surface-hover
                 {autoStartLoaded ? '' : 'cursor-not-allowed opacity-50'}"
        >
          <span class="text-sm text-content">Launch at Login</span>
          <div
            class="relative h-[20px] w-[34px] rounded-full transition-all duration-200
                   {launchAtLoginEnabled ? 'bg-accent' : 'bg-[#E9E9EB] dark:bg-[#39393D]'}"
          >
            <div
              class="absolute top-[2px] h-4 w-4 rounded-full bg-white shadow-sm transition-all duration-200
                     {launchAtLoginEnabled ? 'translate-x-[16px]' : 'translate-x-[2px]'}"
            ></div>
          </div>
        </button>
      </div>
    </section>

    <!-- Theme -->
    <section>
      <h2 class="px-1 pb-1 text-[11px] font-medium text-content-secondary">Theme</h2>
      <div class="overflow-hidden rounded-xl bg-surface-secondary">
        <div class="px-3 py-2.5">
          <div class="flex gap-0.5 rounded-lg bg-surface p-0.5">
            {#each [
              { value: "system", label: "System", Icon: Monitor },
              { value: "light", label: "Light", Icon: Sun },
              { value: "dark", label: "Dark", Icon: Moon },
            ] as opt (opt.value)}
              <button
                onclick={() => selectTheme(opt.value as ThemePreference)}
                class="flex flex-1 items-center justify-center gap-1.5 rounded-md px-2 py-1.5 text-[11px] font-medium transition-colors
                       {currentTheme === opt.value
                         ? 'bg-surface-secondary text-content shadow-sm'
                         : 'text-content-secondary hover:text-content'}"
              >
                <opt.Icon size={13} />
                {opt.label}
              </button>
            {/each}
          </div>
        </div>
      </div>
    </section>

    <!-- Notifications -->
    <section>
      <h2 class="px-1 pb-1 text-[11px] font-medium text-content-secondary">Notifications</h2>
      <div class="overflow-hidden rounded-xl bg-surface-secondary">
        {#each notificationItems as item, index (item.key)}
          {#if index > 0}
            <div class="mx-3 border-t border-border/30"></div>
          {/if}
          <button
            onclick={() => toggleNotification(item.key)}
            class="flex w-full items-center justify-between px-3 py-2.5 text-left transition-colors hover:bg-surface-hover"
          >
            <span class="text-sm text-content">{item.label}</span>
            <div
              class="relative h-[20px] w-[34px] rounded-full transition-all duration-200
                     {(settings as any)[item.key] ? 'bg-accent' : 'bg-[#E9E9EB] dark:bg-[#39393D]'}"
            >
              <div
                class="absolute top-[2px] h-4 w-4 rounded-full bg-white shadow-sm transition-all duration-200
                       {(settings as any)[item.key] ? 'translate-x-[16px]' : 'translate-x-[2px]'}"
              ></div>
            </div>
          </button>
        {/each}
      </div>
    </section>

    <!-- Repositories -->
    <section>
      <div class="flex items-center justify-between gap-2 px-1 pb-1">
        <h2 class="text-[11px] font-medium text-content-secondary">Repositories</h2>
        {#if allRepos.length > 0}
          <button
            onclick={() => allHidden ? showAll() : hideAll()}
            class="text-[11px] font-medium text-accent transition-colors hover:text-accent-hover"
          >
            {allHidden ? "Show all" : "Hide all"}
          </button>
        {/if}
      </div>
      <div class="overflow-hidden rounded-xl bg-surface-secondary">
        {#if allRepos.length === 0}
          <p class="px-3 py-2 text-sm text-content-secondary">No repositories yet</p>
        {:else}
          {#each allRepos as repo, index (repo)}
            {#if index > 0}
              <div class="mx-3 border-t border-border/30"></div>
            {/if}
            <button
              onclick={() => toggleRepo(repo)}
              class="flex w-full items-center gap-2.5 px-3 py-2 text-left transition-colors hover:bg-surface-hover"
            >
              <div
                class="flex h-3.5 w-3.5 shrink-0 items-center justify-center rounded border
                       {settings.hidden_repos.includes(repo)
                         ? 'border-content-tertiary bg-transparent'
                         : 'border-accent bg-accent'}"
              >
                {#if !settings.hidden_repos.includes(repo)}
                  <svg class="h-2.5 w-2.5 text-white" viewBox="0 0 12 12" fill="none">
                    <path d="M2 6l3 3 5-5" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"/>
                  </svg>
                {/if}
              </div>
              <span class="truncate text-sm text-content">{repo}</span>
            </button>
          {/each}
        {/if}
      </div>
    </section>
  {/if}
</div>
