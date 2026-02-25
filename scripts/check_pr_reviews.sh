#!/usr/bin/env bash
# Check for unresolved PR review comments.
# Usage: ./scripts/check_pr_reviews.sh <pr_number>
# Exits 0 if all resolved, 1 if unresolved comments exist.

set -euo pipefail

if [ $# -eq 0 ]; then
  echo "Usage: $0 <pr_number>"
  exit 1
fi

PR_NUMBER="$1"
if ! [[ "$PR_NUMBER" =~ ^[0-9]+$ ]]; then
  echo "❌ PR number must be numeric. Got: '$PR_NUMBER'" >&2
  exit 1
fi

# Resolve repo owner/name
repo_info=$(gh repo view --json owner,name --jq '{owner: .owner.login, name: .name}')
OWNER=$(echo "$repo_info" | jq -r '.owner // empty')
REPO=$(echo "$repo_info" | jq -r '.name // empty')

if [[ -z "$OWNER" || -z "$REPO" ]]; then
  echo "❌ Failed to resolve repository owner/name." >&2
  exit 1
fi

# shellcheck disable=SC2016 # Single quotes are intentional — this is a GraphQL query.
GRAPHQL_QUERY='query($owner: String!, $repo: String!, $pr: Int!, $cursor: String) {
  repository(owner: $owner, name: $repo) {
    pullRequest(number: $pr) {
      reviewThreads(first: 100, after: $cursor) {
        pageInfo {
          hasNextPage
          endCursor
        }
        nodes {
          id
          isResolved
          comments(first: 1) {
            nodes {
              author { login }
              body
            }
          }
        }
      }
    }
  }
}'

UNRESOLVED=""
cursor="null"

while true; do
  page_data=$(gh api graphql \
    -f query="$GRAPHQL_QUERY" \
    -F owner="$OWNER" \
    -F repo="$REPO" \
    -F pr="$PR_NUMBER" \
    -F cursor="$cursor")

  if [ "$(echo "$page_data" | jq -r '.data.repository.pullRequest == null')" = "true" ]; then
    echo "❌ PR #$PR_NUMBER does not exist in ${OWNER}/${REPO}." >&2
    exit 1
  fi

  unresolved_page=$(echo "$page_data" | jq -r '.data.repository.pullRequest.reviewThreads.nodes[]? | select(.isResolved == false) | {thread_id: .id, user: (.comments.nodes[0].author.login // "unknown"), body: (.comments.nodes[0].body // "")}')

  if [[ -n "$unresolved_page" ]]; then
    if [[ -n "$UNRESOLVED" ]]; then
      UNRESOLVED+=$'\n'
    fi
    UNRESOLVED+="$unresolved_page"
  fi

  has_next=$(echo "$page_data" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.hasNextPage')
  end_cursor=$(echo "$page_data" | jq -r '.data.repository.pullRequest.reviewThreads.pageInfo.endCursor // empty')

  if [ "$has_next" != "true" ]; then
    break
  fi

  if [[ -z "$end_cursor" ]]; then
    echo "❌ GraphQL reported hasNextPage=true with empty endCursor" >&2
    exit 1
  fi
  cursor="$end_cursor"
done

if [ -n "$UNRESOLVED" ]; then
  echo "❌ Unresolved review comments found:"
  echo "$UNRESOLVED" | jq -r '"  \(.user): \(.body)"'
  echo ""
  echo "View PR: https://github.com/${OWNER}/${REPO}/pull/$PR_NUMBER"
  exit 1
fi

echo "✅ All review comments resolved"
exit 0
