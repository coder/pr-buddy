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
│   ├── App.svelte                # Root component (auth routing + event listeners)
│   ├── main.ts                   # Entry point, mounts App
│   ├── lib/
│   │   ├── types.ts              # TS interfaces mirroring Rust models (snake_case)
│   │   ├── stores.ts             # groupPrs() + shared writable stores
│   │   ├── AuthScreen.svelte     # GitHub Device Flow login UI
│   │   ├── PRPanel.svelte        # Main panel with grouped sections
│   │   ├── PRSection.svelte      # Collapsible section group
│   │   ├── PRCard.svelte         # Individual PR row
│   │   ├── SettingsPage.svelte   # Notification/theme/repo visibility settings
│   │   ├── UpdateDialog.svelte   # In-app updater UI
│   │   ├── TitleBar.svelte       # Custom frameless title bar controls
│   │   ├── StatusBadge.svelte    # Color-coded status dot
│   │   └── theme.svelte.ts       # Global theme preference + dark-mode toggling
│   ├── __mocks__/                # Tauri API stubs for vitest/jsdom
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
│       ├── models.rs             # Shared data types (PullRequest, PrState, etc.)
│       ├── state.rs              # AppState (Mutex-wrapped shared state)
│       ├── auth.rs               # Device Flow OAuth + token persistence
│       ├── github.rs             # GraphQL client + Tauri commands
│       ├── menu.rs               # Native tray menu construction + section grouping
│       ├── poller.rs             # Background adaptive polling loop
│       ├── notifications.rs      # Event diffing + OS notification dispatch
│       ├── settings.rs           # User settings load/save + IPC commands
│       ├── updater.rs            # Update check/install commands + events
│       └── avatars.rs            # Avatar fetch/cache for tray menu icons
├── scripts/
│   ├── smoke-test.sh             # Vite dev import-resolution smoke test
│   ├── check_pr_reviews.sh       # CI helper: unresolved review thread check
│   ├── check_codex_comments.sh   # CI helper: Codex approval/comment status
│   ├── wait_pr_checks.sh         # Poll helper: wait until checks/mergeability pass
│   ├── wait_pr_codex.sh          # Poll helper: wait for Codex final signal
│   └── generate_icons.py         # Python icon generator (cairosvg + Pillow)
├── package.json
├── Makefile                      # install/dev/build/check/test/smoke/ci/clean targets
├── vite.config.ts
├── vitest.config.ts              # Test config (jsdom + Tauri mocks)
├── svelte.config.js
├── tailwind.config.js
├── postcss.config.js
└── tsconfig.json
```

### Key Files

- **`src-tauri/src/lib.rs`** — App entry point. Registers plugins, tray, IPC commands, menu events, and starts the poller.
- **`src-tauri/src/models.rs`** + **`src/lib/types.ts`** — Canonical cross-layer types. Keep fields and enum values in sync.
- **`src-tauri/src/menu.rs`** + **`src/lib/stores.ts`** — Two implementations of PR grouping (tray menu + Svelte panel). Keep section logic aligned.
- **`src-tauri/src/auth.rs`** — GitHub Device Flow plus persisted token load/save/delete.
- **`src-tauri/src/settings.rs`** — User settings storage (`settings.json`) and settings commands.
- **`src-tauri/src/updater.rs`** + **`src/lib/UpdateDialog.svelte`** — Update check/install flow and download progress events.
- **`src-tauri/src/poller.rs`** — Adaptive polling (30s active / 120s idle) plus periodic update checks.
- **`src-tauri/src/notifications.rs`** — `diff_pr_states()` and notification dispatch (gated by settings).
- **`src-tauri/tauri.conf.json`** — Hidden-by-default tray window (380×520), CSP allowlist, updater endpoint config.
- **`src/App.svelte`** — Root runtime: auth bootstrap, event listeners (`prs-updated`, `auth-cleared`, `open-settings`), and view routing.

### Tauri Commands (IPC boundary)

All commands are registered in `lib.rs` via `invoke_handler`. Frontend calls them with `invoke()` from `@tauri-apps/api/core`:

| Command | Module | Returns |
|---------|--------|---------|
| `start_device_flow_cmd` | auth.rs | `DeviceCodeResponse` |
| `poll_for_token_cmd` | auth.rs | `bool` |
| `logout_cmd` | auth.rs | `()` |
| `is_authenticated_cmd` | auth.rs | `bool` |
| `get_pull_requests_cmd` | github.rs | `Vec<PullRequest>` |
| `get_user_info_cmd` | github.rs | `Option<GitHubUser>` |
| `refresh_prs_cmd` | github.rs | `Vec<PullRequest>` |
| `get_settings_cmd` | settings.rs | `UserSettings` |
| `save_settings_cmd` | settings.rs | `()` |
| `check_for_update_cmd` | updater.rs | `UpdateCheckResult` |
| `install_update_cmd` | updater.rs | `()` |

### Tauri Events

| Event | Direction | Payload |
|-------|-----------|---------|
| `prs-updated` | Rust → Frontend | `PullRequest[]` |
| `auth-cleared` | Rust → Frontend | `()` |
| `open-settings` | Rust → Frontend | `()` |
| `update-download-progress` | Rust → Frontend | `{ chunk_length, content_length }` |

### Linux Prerequisites

The system tray requires `libayatana-appindicator3`. Install it before running `make dev` or the release binary:

| Distro | Command |
|--------|---------|
| Arch / Manjaro | `sudo pacman -S libayatana-appindicator` |
| Ubuntu / Debian | `sudo apt install libayatana-appindicator3-dev` |
| Fedora | `sudo dnf install libayatana-appindicator-gtk3` |

The `.deb` and `.rpm` bundles declare this as a package dependency so it installs automatically when users install through those package formats. Arch/pacman users must install it manually.

## Essential Commands

```bash
# Install frontend dependencies (required before any other command)
npm install         # or: make install

# Full desktop dev loop (Tauri backend + Vite frontend)
make dev            # or: npm run dev

# Frontend-only dev server (no Rust compilation)
npm run vite:dev

# Production builds
make build          # or: npm run build
npm run vite:build  # frontend-only production bundle

# Type-check + tests
make check          # or: npm run check
make test           # or: npm run test
make smoke          # or: npm run smoke

# Rust checks (run when touching src-tauri)
(cd src-tauri && cargo check)

# Formatting
(cd src-tauri && cargo fmt)
# Frontend: no dedicated formatter script is configured yet.

# Linting
(cd src-tauri && cargo clippy -- -D warnings)
# Frontend: no ESLint script is configured yet.

# CI shortcut (check + test + smoke + build)
make ci

# PR workflow helper scripts
./scripts/check_pr_reviews.sh <pr_number>
./scripts/check_codex_comments.sh <pr_number>
./scripts/wait_pr_checks.sh <pr_number>
./scripts/wait_pr_codex.sh <pr_number>

# Regenerate app and tray icons (requires Python 3, cairosvg, Pillow)
python3 scripts/generate_icons.py

# Clean build artifacts
make clean
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

### Frontend ↔ Tray Section Parity

PR grouping logic exists in **two places**:
- `src/lib/stores.ts` (`groupPrs()` for the Svelte panel)
- `src-tauri/src/menu.rs` (`group_prs()` for the native tray menu)

When adding/removing/renaming a section, update both implementations in the same change.
Also update assertions in `src/lib/components.test.ts` so section behavior stays covered.

### Interpreting User Requests (Tray-First Default)

This app has two UI surfaces:
- **Native tray menu** (Rust): `src-tauri/src/menu.rs`
- **Svelte webview panel/windows**: `src/App.svelte` + `src/lib/*.svelte`

When a user asks for a "UI", "menu", "section", or "status" change **without naming a surface**, assume they mean the **native tray menu first**. Most day-to-day usability is in the tray.

Only treat a request as webview-only when the user explicitly references panel/window behavior (for example: settings page, updater dialog, title bar, Auth screen, PR cards).

If both surfaces are plausible, either:
1. ask a clarifying question, or
2. state the assumption explicitly and implement tray-first.

### Tauri Command Pattern

Rust commands use `#[tauri::command]` and return `Result<T, E>` where `E` is serializable (`AuthError` for auth/github commands, `String` for settings/updater). Commands that need shared state access it via `State<'_, AppState>`:

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

### Icons (Lucide)

Use **`@lucide/svelte`** (the Svelte 5 package), **not** `lucide-svelte` (Svelte 4 only).

Always use deep imports — barrel imports cause Vite dev server resolution failures:

```typescript
// ✅ Correct — deep import
import Bell from "@lucide/svelte/icons/bell";

// ❌ Wrong — barrel import, breaks Vite dev
import { Bell } from "@lucide/svelte";

// ❌ Wrong — old Svelte 4 package
import { Bell } from "lucide-svelte";
```

Icon names are kebab-case: `bell`, `x-circle`, `rotate-ccw`, `check-circle`, `file-edit`, `git-merge`, `log-out`, `party-popper`, `refresh-cw`, etc.

**When adding a new icon import**, also add it to the `optimizeDeps.include` array in `vite.config.ts` so Vite pre-bundles it for dev.

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
- **Do not update PR section grouping in only one layer.** `src/lib/stores.ts` and `src-tauri/src/menu.rs` must stay in sync, and `src/lib/components.test.ts` should be updated when section behavior changes.
- **Do not assume ambiguous UX requests target the Svelte webview.** Default to tray-menu changes unless the user clearly asks for panel/window behavior.
- **Do not pass arguments to `TrayIconBuilder::new()`.** Tauri v2.10 takes zero arguments. Use `.icon()` to set the icon separately.
- **Do not use `async` `onMount` callbacks that return cleanup functions** in Svelte 5 — it causes type errors. See the pattern above.
- **Do not use the bundle identifier `com.prbuddy.app`.** The `.app` suffix conflicts with macOS bundle extensions. Current identifier: `com.prbuddy.dev`.
- **Do not commit `node_modules/`, `dist/`, or `src-tauri/target/`.** These are in `.gitignore`.

## Testing

### Test Stack

| Tool | Purpose |
|------|---------|
| Vitest | Test runner (uses Vite for transforms) |
| @testing-library/svelte | Component rendering in jsdom |
| jsdom | Browser environment for Node |
| scripts/smoke-test.sh | Dev server import validation |

### Component Tests (`src/lib/components.test.ts`)

Every Svelte component has a basic render test that imports it, mounts it with mock props, and verifies it produces output. This catches:
- Broken import paths (e.g., wrong icon package)
- Missing exports
- Render crashes from bad props

Run with `make test` or `npm run test`.

### Smoke Test (`scripts/smoke-test.sh`)

Starts the Vite dev server, fetches every entry module, and checks for `"Failed to resolve import"` errors in the server log. This catches issues that `vite build` and `svelte-check` miss — specifically, import resolution differences between Vite's build and dev modes (e.g., the `"svelte"` export condition in `@lucide/svelte`).

Run with `make smoke` or `npm run smoke`.

### Tauri API Mocks (`src/__mocks__/`)

Components import Tauri APIs (`@tauri-apps/api/core`, `@tauri-apps/plugin-opener`, etc.) which don't exist in a jsdom environment. The `vitest.config.ts` aliases these to stub modules in `src/__mocks__/` that return sensible defaults.

When adding a new Tauri plugin import to a component, add a corresponding mock in `src/__mocks__/` and alias it in `vitest.config.ts`.

### CI Shortcut

`make ci` runs the full validation suite in order: `check → test → smoke → build`. Run this before pushing.

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

1. Run `make ci` — this runs type-check, unit tests, smoke test, and production build in sequence.
2. Alternatively, run them individually: `npm run check`, `npm run test`, `npm run smoke`, `npm run vite:build`.
3. If Rust code changed and Rust 1.77+ is available, run `(cd src-tauri && cargo fmt && cargo clippy -- -D warnings && cargo check)`.
4. Do not push to `origin/main` or `origin/master` directly.
5. Branch names must be prefixed with `mike/` (e.g., `mike/fix-tray-click`).
6. **Do not claim imports or dependencies work without running `make smoke`** — `vite build` and `svelte-check` do not catch all import resolution errors that appear in the Vite dev server.

### Pull Request Descriptions

- Summarise what changed and why.
- List files modified by category (Rust backend / Svelte frontend / config).
- Note any build verification results (`npm run check`, `npm run vite:build`, `cargo check`).

### Codex Review Comments

Every PR with Codex reviews enabled must have **zero unresolved review comments** before merging. Follow this workflow:

1. After pushing, poll for new Codex review comments (`gh api repos/{owner}/{repo}/pulls/{number}/comments`).
2. For each new comment, fix the issue in code, commit, and push.
3. Reply to each comment explaining the fix, then resolve the review thread via the GraphQL `resolveReviewThread` mutation.
4. After pushing fixes, **re-request a review** by commenting `@codex review` on the PR (`gh pr comment {number} --body "@codex review"`). Do not rely on the push alone to trigger a new review pass.
5. **Wait deterministically for Codex to confirm.** Poll the PR timeline (~2 min intervals) until Codex signals approval — either a 👍 reaction on your comment, an approving review (`APPROVED` state), or a message indicating no issues remain. Do not proceed on a timeout or silence; you must receive an explicit positive signal.
6. If Codex posts new comments instead of approving, go back to step 2 and repeat.
7. Only consider the PR ready to merge when Codex has explicitly approved and all review threads are resolved.

