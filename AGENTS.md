You are an experienced, pragmatic software engineering AI agent. Do not over-engineer a solution when a simple one is possible. Keep edits minimal. If you want an exception to ANY rule, you MUST stop and get permission first.

# PR Buddy

## Project Overview

PR Buddy is a lightweight, cross-platform system tray desktop app that monitors your GitHub pull requests and sends native OS notifications for key state changes (checks failed, merge queue removal, merges, checks passed). Clicking the tray icon opens a compact panel listing your active PRs from the last 14 days, grouped by status.

**Architecture:** Tauri v2 desktop app — a Rust backend handles GitHub API polling, OAuth, event diffing, and notifications; a Svelte 5 frontend renders the tray panel UI. Communication between layers uses Tauri IPC commands and events.

### Tech Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| App shell | Tauri | v2.10 |
| Frontend | Svelte | v5 (runes syntax) |
| Frontend lang | TypeScript | strict mode |
| Styling | Tailwind CSS | v3 |
| Build tool | Vite | v6 |
| Backend | Rust | edition 2021, requires 1.77+ |
| HTTP client | reqwest | 0.12 |
| Auth | GitHub OAuth Device Flow | — |
| API | GitHub GraphQL | search + PR fragments |
| Notifications | tauri-plugin-notification | 2.3 |
| Secure storage | tauri-plugin-stronghold | 2.3 |

## Reference

### Project Structure

```
pr-buddy/
├── src/                          # Svelte 5 frontend
│   ├── App.svelte                # Root component (auth routing, event listener)
│   ├── main.ts                   # Entry point, mounts App
│   ├── lib/
│   │   ├── types.ts              # TS interfaces mirroring Rust models (snake_case)
│   │   ├── stores.ts             # Svelte stores + groupPrs() section logic
│   │   ├── AuthScreen.svelte     # GitHub Device Flow login UI
│   │   ├── PRPanel.svelte        # Main panel with grouped sections
│   │   ├── PRSection.svelte      # Collapsible section group
│   │   ├── PRCard.svelte         # Individual PR row
│   │   └── StatusBadge.svelte    # Color-coded status dot
│   └── styles/
│       └── app.css               # Tailwind directives + dark theme overrides
├── src-tauri/                    # Rust backend
│   ├── Cargo.toml
│   ├── tauri.conf.json           # Window config, CSP, tray, bundle icons
│   ├── capabilities/default.json # Tauri v2 ACL permissions
│   ├── build.rs
│   ├── icons/                    # App + tray icons (generated via scripts/)
│   └── src/
│       ├── main.rs               # Desktop entry: calls lib::run()
│       ├── lib.rs                # Tauri setup: plugins, tray, commands, poller
│       ├── models.rs             # Data types: PullRequest, PrState, CheckStatus, etc.
│       ├── state.rs              # AppState (Mutex-wrapped shared state)
│       ├── auth.rs               # Device Flow OAuth commands
│       ├── github.rs             # GraphQL client + Tauri commands
│       ├── poller.rs             # Background adaptive polling loop
│       └── notifications.rs      # Event diffing + OS notification dispatch
├── scripts/
│   └── generate_icons.py         # Python3+Pillow icon generator (idempotent)
├── package.json
├── vite.config.ts
├── svelte.config.js
├── tailwind.config.js
├── postcss.config.js
└── tsconfig.json
```

### Key Files

- **`src-tauri/src/lib.rs`** — App entry point. Registers all plugins, builds the tray icon, wires Tauri commands, starts the background poller.
- **`src-tauri/src/models.rs`** — Canonical data types shared across backend. Frontend types in `src/lib/types.ts` must stay in sync (use `snake_case` field names, matching Rust's `#[serde(rename_all = "lowercase")]`).
- **`src-tauri/src/auth.rs`** — GitHub Device Flow. Client ID is hardcoded with env var override (`GITHUB_CLIENT_ID`).
- **`src-tauri/src/poller.rs`** — Adaptive polling: 30s when PRs have pending checks or are in merge queue, 120s otherwise.
- **`src-tauri/src/notifications.rs`** — `diff_pr_states()` compares old/new PR snapshots to detect state transitions.
- **`src-tauri/tauri.conf.json`** — Window is hidden by default (tray-only), 380×520px, no decorations, `alwaysOnTop`.
- **`src/lib/stores.ts`** — `groupPrs()` categorises PRs into ordered sections (merge queue → failing → changes requested → waiting → approved → draft → merged).
- **`src/App.svelte`** — Root component. Handles auth check, Tauri event subscription (`prs-updated`), routing between AuthScreen and PRPanel.

### Tauri Commands (IPC boundary)

All commands are registered in `lib.rs` via `invoke_handler`. Frontend calls them with `invoke()` from `@tauri-apps/api/core`:

| Command | Module | Returns |
|---------|--------|---------|
| `start_device_flow_cmd` | auth.rs | `DeviceCodeResponse` |
| `poll_for_token_cmd` | auth.rs | `bool` |
| `logout_cmd` | auth.rs | `()` |
| `is_authenticated_cmd` | auth.rs | `bool` |
| `get_pull_requests_cmd` | github.rs | `Vec<PullRequest>` |
| `get_user_info_cmd` | github.rs | `GitHubUser` |
| `refresh_prs_cmd` | github.rs | `Vec<PullRequest>` |

### Tauri Events

| Event | Direction | Payload |
|-------|-----------|---------|
| `prs-updated` | Rust → Frontend | `PullRequest[]` |

## Essential Commands

```bash
# Install frontend dependencies (required before any other command)
npm install

# Full Tauri app — Rust backend + Vite frontend (requires Rust 1.77+)
npm run dev

# Build release binary (output in src-tauri/target/release/bundle/)
npm run build

# Frontend-only dev server (no Rust compilation, for UI iteration)
npm run vite:dev

# Frontend production build only
npm run vite:build

# TypeScript + Svelte type checking
npm run check

# Regenerate app icons (requires Python 3 + Pillow)
python3 scripts/generate_icons.py

# Clean build artifacts
rm -rf dist/ src-tauri/target/
```

**Important:** `npm run dev` and `npm run build` invoke `tauri dev`/`tauri build`, which internally run `npm run vite:dev`/`npm run vite:build` via `beforeDevCommand`/`beforeBuildCommand` in `tauri.conf.json`. Do **not** change the top-level `dev`/`build` scripts to call Vite directly — that breaks the Tauri build chain. Do **not** point `beforeBuildCommand` at `npm run build` — that creates an infinite recursion loop.

## Patterns

### Rust ↔ Frontend Type Sync

Rust models in `src-tauri/src/models.rs` use `#[serde(rename_all = "lowercase")]` for enums. TypeScript interfaces in `src/lib/types.ts` must use matching `snake_case` field names and lowercase string union values:

```rust
// Rust (models.rs)
#[serde(rename_all = "lowercase")]
pub enum PrState { Open, Closed, Merged }
```
```typescript
// TypeScript (types.ts)
export type PrState = "open" | "closed" | "merged";
```

When adding a field to `PullRequest`, update **both** `models.rs` and `types.ts`.

### Tauri Command Pattern

Rust commands use `#[tauri::command]` and return `Result<T, AuthError>`. The `AuthError` type implements `Serialize` so Tauri can pass errors to the frontend. Commands access shared state via `State<'_, AppState>`:

```rust
#[tauri::command]
pub async fn my_command(state: State<'_, AppState>) -> Result<MyData, AuthError> {
    let token = state.token.lock().unwrap();
    // ...
}
```

New commands must be added to the `invoke_handler` macro in `lib.rs`.

### Svelte 5 Runes

This project uses Svelte 5 runes syntax (`$state`, `$derived`, `$effect`, `$props`), **not** Svelte 4 stores in components. The `src/lib/stores.ts` file uses classic `writable()` stores for backward compat, but components use runes directly.

### onMount Async

Svelte 5's `onMount` does **not** accept async callbacks that return cleanup functions. Use a synchronous `onMount` callback and call a separate `async function init()` inside:

```svelte
onMount(() => {
  void init();     // fire-and-forget async
});
onDestroy(() => { /* cleanup */ });
```

Do **not** write `onMount(async () => { ... return cleanup; })`.

## Anti-Patterns

- **Do not change `npm run dev`/`npm run build` to call Vite.** These must call `tauri dev`/`tauri build`. Frontend-only scripts are `vite:dev`/`vite:build`.
- **Do not pass arguments to `TrayIconBuilder::new()`.** Tauri v2.10 takes zero arguments. Use `.icon()` to set the icon separately.
- **Do not use `async` `onMount` callbacks that return cleanup functions** in Svelte 5 — it causes type errors. See the pattern above.
- **Do not use the bundle identifier `com.prbuddy.app`.** The `.app` suffix conflicts with macOS bundle extensions. Current identifier: `com.prbuddy.dev`.
- **Do not commit `node_modules/`, `dist/`, or `src-tauri/target/`.** These are in `.gitignore`.

## Commit and Pull Request Guidelines

### Commit Conventions

This project uses `type: message` commit format. Read `git log --oneline` for examples:

```
feat: add Rust backend modules for PR monitoring
fix: bump vite plugin svelte to v5 for vite 6
chore: ignore Cargo.lock
```

Common prefixes: `feat`, `fix`, `chore`, `refactor`, `docs`.

### Before Committing

1. Run `npm run check` — must report 0 errors.
2. Run `npm run vite:build` — must succeed.
3. If Rust code changed and Rust 1.77+ is available, run `cargo check` in `src-tauri/`.
4. Do not push to `origin/main` or `origin/master` directly.
5. Branch names must be prefixed with `mike/` (e.g., `mike/fix-tray-click`).

### Pull Request Descriptions

- Summarise what changed and why.
- List files modified by category (Rust backend / Svelte frontend / config).
- Note any build verification results (`npm run check`, `npm run vite:build`, `cargo check`).
