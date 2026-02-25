<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl } from "@tauri-apps/plugin-opener";
  import { Bell } from "lucide-svelte";
  import type { DeviceCodeResponse } from "./types";

  interface Props {
    onSuccess: () => void;
  }
  let { onSuccess }: Props = $props();

  let step = $state<"idle" | "waiting" | "error">("idle");
  let deviceResponse = $state<DeviceCodeResponse | null>(null);
  let secondsLeft = $state(0);
  let copied = $state(false);
  let errorMsg = $state("");
  let pollError = $state("");

  let timerInterval: ReturnType<typeof setInterval> | undefined;
  let pollInterval: ReturnType<typeof setInterval> | undefined;

  async function startFlow() {
    step = "waiting";
    errorMsg = "";
    pollError = "";
    try {
      console.log("[auth] Starting device flow...");
      deviceResponse = await invoke<DeviceCodeResponse>("start_device_flow_cmd");
      console.log("[auth] Device flow response:", JSON.stringify(deviceResponse));
      secondsLeft = deviceResponse.expires_in;

      timerInterval = setInterval(() => {
        secondsLeft--;
        if (secondsLeft <= 0) {
          cleanup();
          step = "error";
          errorMsg = "Code expired. Please try again.";
        }
      }, 1000);

      const interval = (deviceResponse.interval || 5) * 1000;
      console.log("[auth] Starting poll every", interval, "ms");
      pollInterval = setInterval(async () => {
        console.log("[auth] Polling for token...");
        try {
          const success = await invoke<boolean>("poll_for_token_cmd", {
            deviceCode: deviceResponse!.device_code,
          });
          console.log("[auth] Poll result:", success);
          pollError = "";
          if (success) {
            console.log("[auth] Authenticated!");
            cleanup();
            onSuccess();
          }
        } catch (e) {
          const msg = typeof e === "string" ? e : JSON.stringify(e);
          console.error("[auth] Poll error:", msg);
          pollError = msg;
        }
      }, interval);
    } catch (e) {
      const msg = typeof e === "string" ? e : JSON.stringify(e);
      console.error("[auth] Failed to start device flow:", msg);
      step = "error";
      errorMsg = msg || "Failed to start sign in. Please try again.";
    }
  }

  function cleanup() {
    if (timerInterval) clearInterval(timerInterval);
    if (pollInterval) clearInterval(pollInterval);
    timerInterval = undefined;
    pollInterval = undefined;
  }

  async function copyCode() {
    if (deviceResponse) {
      await navigator.clipboard.writeText(deviceResponse.user_code);
      copied = true;
      setTimeout(() => { copied = false; }, 2000);
    }
  }

  async function openGitHub() {
    if (deviceResponse) {
      await openUrl(deviceResponse.verification_uri);
    }
  }

  function formatTime(secs: number): string {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${m}:${s.toString().padStart(2, "0")}`;
  }
</script>

<div class="flex flex-col items-center justify-center h-full px-8">
  <div class="flex flex-col items-center gap-2 mb-8">
    <Bell size={28} class="text-gray-400" />
    <h1 class="text-xl font-bold text-white">PR Buddy</h1>
    <p class="text-gray-500 text-sm">Stay on top of your pull requests</p>
  </div>

  {#if step === "idle"}
    <button
      onclick={startFlow}
      class="w-full max-w-[280px] flex items-center justify-center gap-2 bg-white text-black font-semibold
             py-2.5 px-5 rounded-lg hover:bg-gray-200 transition-colors"
    >
      <svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
        <path d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205
          11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555
          -3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02
          -.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305
          3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925
          0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005
          -.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295
          -1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23
          1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81
          1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825
          .57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"/>
      </svg>
      Sign in with GitHub
    </button>

  {:else if step === "waiting" && deviceResponse}
    <div class="flex flex-col items-center gap-5 w-full max-w-[300px]">
      <p class="text-gray-400 text-sm text-center">Enter this code on GitHub:</p>

      <div class="bg-[#1e1e2e] border border-gray-700 rounded-xl py-4 px-6 w-full text-center">
        <span class="text-2xl font-mono font-bold tracking-[0.3em] text-white">
          {deviceResponse.user_code}
        </span>
      </div>

      <div class="flex gap-2 w-full">
        <button
          onclick={copyCode}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium transition-colors
                 {copied ? 'bg-green-600 text-white' : 'bg-[#1e1e2e] text-gray-300 hover:bg-[#2a2a3e]'}"
        >
          {copied ? "✓ Copied" : "Copy Code"}
        </button>
        <button
          onclick={openGitHub}
          class="flex-1 py-2 px-3 rounded-lg text-sm font-medium bg-purple-600 text-white
                 hover:bg-purple-700 transition-colors"
        >
          Open GitHub →
        </button>
      </div>

      <div class="flex flex-col items-center gap-1">
        <div class="w-5 h-5 border-2 border-purple-500 border-t-transparent rounded-full animate-spin"></div>
        <p class="text-gray-500 text-xs">Waiting for authorization…</p>
        <p class="text-gray-600 text-xs">Expires in {formatTime(secondsLeft)}</p>
        {#if pollError}
          <p class="text-red-400 text-xs mt-1 text-center px-2">{pollError}</p>
        {/if}
      </div>
    </div>

  {:else if step === "error"}
    <div class="flex flex-col items-center gap-4">
      <p class="text-red-400 text-sm">{errorMsg}</p>
      <button
        onclick={() => { step = "idle"; }}
        class="py-2 px-5 rounded-lg text-sm font-medium bg-[#1e1e2e] text-gray-300
               hover:bg-[#2a2a3e] transition-colors"
      >
        Try Again
      </button>
    </div>
  {/if}
</div>
