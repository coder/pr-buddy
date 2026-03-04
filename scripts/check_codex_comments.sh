#!/usr/bin/env bash
# Check for Codex review bot comments on a PR.
# Usage: ./scripts/check_codex_comments.sh <pr_number>
#
# Exits:
#   0 - Codex approved (thumbs-up reaction or approval comment)
#   1 - Codex left comments to address or hit rate limits
#  10 - Codex hasn't responded yet

set -euo pipefail

if [ $# -eq 0 ]; then
  echo "Usage: $0 <pr_number>"
  exit 1
fi

PR_NUMBER=$1
if ! [[ "$PR_NUMBER" =~ ^[0-9]+$ ]]; then
  echo "❌ PR number must be numeric. Got: '$PR_NUMBER'" >&2
  exit 1
fi

BOT_LOGIN="chatgpt-codex-connector"
CODEX_APPROVAL_REGEX="Didn't find any major issues"
CODEX_RATE_LIMIT_REGEX="usage limits have been reached"

# Resolve repo owner/name
repo_info=$(gh repo view --json owner,name --jq '{owner: .owner.login, name: .name}')
OWNER=$(echo "$repo_info" | jq -r '.owner // empty')
REPO=$(echo "$repo_info" | jq -r '.name // empty')

if [[ -z "$OWNER" || -z "$REPO" ]]; then
  echo "❌ Failed to resolve repository owner/name." >&2
  exit 1
fi

# shellcheck disable=SC2016 # Single quotes are intentional — this is a GraphQL query.
GRAPHQL_QUERY='query($owner: String!, $repo: String!, $pr: Int!) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $pr) {
      state
      comments(last: 100) {
        nodes {
          id
          author { login }
          body
          createdAt
          isMinimized
        }
      }
      reviewThreads(last: 100) {
        nodes {
          id
          isResolved
          comments(first: 1) {
            nodes {
              author { login }
              body
              createdAt
            }
          }
        }
      }
      reactions(last: 100, content: THUMBS_UP) {
        nodes {
          createdAt
          user { login }
        }
      }
      commits(last: 1) {
        nodes {
          commit { pushedDate }
        }
      }
    }
  }
}'

data=$(gh api graphql \
  -f query="$GRAPHQL_QUERY" \
  -F owner="$OWNER" \
  -F repo="$REPO" \
  -F pr="$PR_NUMBER")

if [ "$(echo "$data" | jq -r '.data.repository.pullRequest == null')" = "true" ]; then
  echo "❌ PR #$PR_NUMBER does not exist in ${OWNER}/${REPO}." >&2
  exit 1
fi

# Check for thumbs-up reaction from Codex bot (must be after latest push)
last_push=$(echo "$data" | jq -r \
  '.data.repository.pullRequest.commits.nodes[-1].commit.pushedDate // empty')

has_thumbsup=$(echo "$data" | jq -r --arg bot "$BOT_LOGIN" --arg since "${last_push:-}" \
  '[.data.repository.pullRequest.reactions.nodes[]?
    | select(.user.login == $bot)
    | select($since == "" or .createdAt > $since)
  ] | length')

if [ "$has_thumbsup" -gt 0 ]; then
  echo "✅ Codex approved (👍 reaction on PR description)"
  exit 0
fi

# Check regular comments from Codex bot
bot_comments=$(echo "$data" | jq -r --arg bot "$BOT_LOGIN" \
  '[.data.repository.pullRequest.comments.nodes[]? | select(.author.login == $bot and .isMinimized == false)]')
bot_comment_count=$(echo "$bot_comments" | jq 'length')

# Check review thread comments from Codex bot
bot_threads=$(echo "$data" | jq -r --arg bot "$BOT_LOGIN" \
  '[.data.repository.pullRequest.reviewThreads.nodes[]? | select(.isResolved == false) | select(.comments.nodes[0].author.login == $bot)]')
bot_thread_count=$(echo "$bot_threads" | jq 'length')

# No Codex activity at all — still waiting
if [ "$bot_comment_count" -eq 0 ] && [ "$bot_thread_count" -eq 0 ]; then
  echo "⏳ No Codex response yet on PR #$PR_NUMBER"
  exit 10
fi

# Check if latest comment is an approval
if [ "$bot_comment_count" -gt 0 ]; then
  latest_body=$(echo "$bot_comments" | jq -r '.[-1].body')

  if echo "$latest_body" | grep -q "$CODEX_APPROVAL_REGEX"; then
    echo "✅ Codex approved (approval comment found)"
    exit 0
  fi

  if echo "$latest_body" | grep -q "$CODEX_RATE_LIMIT_REGEX"; then
    echo "❌ Codex hit rate limits. Re-request review later."
    exit 1
  fi
fi

# If there are unresolved review threads from Codex, that's comments to address
if [ "$bot_thread_count" -gt 0 ]; then
  echo "❌ Codex left $bot_thread_count unresolved review comment(s):"
  echo "$bot_threads" | jq -r '.[].comments.nodes[0] | "  \(.body)"'
  echo ""
  echo "View PR: https://github.com/${OWNER}/${REPO}/pull/$PR_NUMBER"
  exit 1
fi

# Bot commented but it's not an approval — treat as comments to address
echo "❌ Codex left comments that may need attention."
echo "$bot_comments" | jq -r '.[-1].body' | head -5
exit 1
