#!/usr/bin/env bash
set -euo pipefail

# Wait for Codex to respond to a `@codex review` request.
#
# Usage: ./scripts/wait_pr_codex.sh <pr_number> [--once]
#
# Exits:
#   0 - Codex approved (thumbs-up on PR description or explicit approval comment)
#   1 - Codex left comments to address OR failed to review (e.g. rate limit)
#  10 - still waiting for Codex response (only in --once mode)

if [ $# -lt 1 ] || [ $# -gt 2 ]; then
  echo "Usage: $0 <pr_number> [--once]"
  exit 1
fi

PR_NUMBER=$1
MODE="wait"

if [ $# -eq 2 ]; then
  if [ "$2" = "--once" ]; then
    MODE="once"
  else
    echo "❌ Unknown argument: '$2'" >&2
    echo "Usage: $0 <pr_number> [--once]" >&2
    exit 1
  fi
fi

POLL_INTERVAL_SECS=30

SCRIPT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
CHECK_CODEX_SCRIPT="$SCRIPT_DIR/check_codex_comments.sh"

if ! [[ "$PR_NUMBER" =~ ^[0-9]+$ ]]; then
  echo "❌ PR number must be numeric. Got: '$PR_NUMBER'" >&2
  exit 1
fi

if [ ! -x "$CHECK_CODEX_SCRIPT" ]; then
  echo "❌ Missing executable helper script: $CHECK_CODEX_SCRIPT" >&2
  exit 1
fi

CHECK_CODEX_ONCE() {
  local rc
  if "$CHECK_CODEX_SCRIPT" "$PR_NUMBER"; then
    return 0
  else
    rc=$?
    return "$rc"
  fi
}

if [ "$MODE" = "once" ]; then
  if CHECK_CODEX_ONCE; then
    rc=0
  else
    rc=$?
  fi

  case "$rc" in
    0 | 1 | 10)
      exit "$rc"
      ;;
    *)
      echo "❌ Unexpected codex check status code '$rc'" >&2
      exit 1
      ;;
  esac
fi

echo "⏳ Waiting for Codex review on PR #$PR_NUMBER..."
echo ""

while true; do
  if CHECK_CODEX_ONCE; then
    rc=0
  else
    rc=$?
  fi

  case "$rc" in
    0)
      exit 0
      ;;
    1)
      exit 1
      ;;
    10)
      echo -ne "\r⏳ Waiting for Codex response...  "
      sleep "$POLL_INTERVAL_SECS"
      ;;
    *)
      echo "❌ Unexpected codex check status code '$rc'" >&2
      exit 1
      ;;
  esac
done
