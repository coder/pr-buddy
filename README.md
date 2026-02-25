# PR Buddy — A lightweight system tray companion that monitors your GitHub PRs

PR Buddy is a fast, native desktop companion for engineers who want immediate insight into pull request health without living in a browser tab.

It runs in your system tray, watches your assigned and authored PRs, and keeps you updated with focused notifications when state changes matter.

## Features

- 🔔 Native OS notifications for: checks failed, merge queue removal, PR merged, checks passed
- 📋 Compact tray panel with grouped PR sections (merge queue, failing checks, needs review, approved, draft, merged)
- 🔐 GitHub OAuth Device Flow — no server needed
- 🖥️ Cross-platform: macOS status bar + Windows taskbar
- ⚡ Tiny binary (~5MB) — built with Tauri v2 + Rust
- 🔄 Adaptive polling (30s when checks pending, 2min when stable)

## Screenshot

<!-- Screenshot coming soon -->

## Tech Stack

| Technology | Role in PR Buddy |
| --- | --- |
| Tauri v2 | Native shell, system tray integration, packaging |
| Svelte 5 | Reactive tray panel UI |
| TypeScript | Frontend application logic and types |
| Tailwind CSS | Utility-first styling for compact UI |
| Rust | Native backend, polling, notifications, OAuth flow |
| GitHub GraphQL API | Pull request and review/check metadata |

## Getting Started

### Prerequisites

- Node.js 18+
- Rust 1.77+
- Tauri CLI

Install Tauri CLI (if needed):

```bash
cargo install tauri-cli --version "^2.0"
```

### Installation

```bash
git clone <your-fork-or-repo-url>
cd pr-buddy
npm install
```

### Run in Development Mode

```bash
npm install && npm run dev
```

### Build for Production

```bash
npm run build
```

### GitHub OAuth Setup

1. Create a GitHub OAuth App in your GitHub account settings.
2. Copy the generated Client ID.
3. Set `GITHUB_CLIENT_ID` in your environment before launching the app.

```bash
export GITHUB_CLIENT_ID="your_client_id_here"
```

PowerShell:

```powershell
$env:GITHUB_CLIENT_ID = "your_client_id_here"
```

## Architecture

PR Buddy follows a lightweight desktop architecture:

1. **Tray icon** provides always-on access from the OS menu bar/taskbar.
2. **Svelte panel** renders grouped PR states and user actions.
3. **Rust backend** handles polling, OAuth device flow, notifications, and state transitions.
4. **GitHub API** is queried via GraphQL for PR status, checks, merge queue state, and review metadata.

Data flows from GitHub into the Rust core, then into the Svelte UI via Tauri commands/events for a responsive tray experience.

## Project Structure

```text
pr-buddy/
├── src/                # Svelte 5 frontend source
│   ├── lib/            # Shared UI/state utilities
│   └── routes/         # App routes/views
├── src-tauri/
│   ├── src/            # Rust backend source
│   ├── icons/          # App and tray icons
│   └── tauri.conf.json # Tauri configuration
├── package.json        # Node scripts and frontend dependencies
└── README.md
```

## License

Licensed under the ISC License. See [LICENSE](./LICENSE) for details.
