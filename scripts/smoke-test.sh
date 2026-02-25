#!/usr/bin/env bash
# Smoke test: starts the Vite dev server, loads all entry modules, and checks
# for import-resolution errors that only surface at dev-serve time (not during
# `vite build`). Exits 0 if clean, 1 if errors found.
set -euo pipefail

PORT=${SMOKE_PORT:-1421}
LOG=$(mktemp)
VITE_PID=""

cleanup() {
  [ -n "$VITE_PID" ] && kill "$VITE_PID" 2>/dev/null || true
  wait "$VITE_PID" 2>/dev/null || true
  rm -f "$LOG"
}
trap cleanup EXIT

echo "Starting Vite dev server on port $PORT..."
npx vite --port "$PORT" --strictPort >"$LOG" 2>&1 &
VITE_PID=$!

# Wait for the server to be ready (up to 15 s)
for i in $(seq 1 30); do
  if curl -sf "http://localhost:$PORT/" >/dev/null 2>&1; then
    break
  fi
  if ! kill -0 "$VITE_PID" 2>/dev/null; then
    echo "FAIL: Vite exited early"
    cat "$LOG"
    exit 1
  fi
  sleep 0.5
done

if ! curl -sf "http://localhost:$PORT/" >/dev/null 2>&1; then
  echo "FAIL: Vite dev server didn't start within 15 s"
  cat "$LOG"
  exit 1
fi

echo "Server ready. Loading entry modules..."

# Fetch the root page (triggers full module graph)
curl -sf "http://localhost:$PORT/" >/dev/null

# Touch key modules that have third-party imports
MODULES=(
  "src/main.ts"
  "src/App.svelte"
  "src/lib/stores.ts"
  "src/lib/PRPanel.svelte"
  "src/lib/PRSection.svelte"
  "src/lib/PRCard.svelte"
  "src/lib/AuthScreen.svelte"
  "src/lib/StatusBadge.svelte"
)

for mod in "${MODULES[@]}"; do
  curl -sf "http://localhost:$PORT/$mod" >/dev/null 2>&1 || true
done

# Give Vite a moment to log any transform errors
sleep 2

# Check for failures
if grep -qiE "Failed to resolve import|Pre-transform error|Internal server error" "$LOG"; then
  echo ""
  echo "FAIL: Import resolution errors detected:"
  echo "------"
  grep -iE "Failed to resolve import|Pre-transform error|Internal server error" "$LOG"
  echo "------"
  exit 1
fi

echo "PASS: No import resolution errors."
