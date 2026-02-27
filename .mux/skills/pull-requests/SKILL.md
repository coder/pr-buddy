---
name: pull-requests
description: >
  Pull request submission and review lifecycle. Use when creating, pushing,
  or monitoring PRs to ensure all CI checks pass and Codex review comments
  are resolved before declaring a PR ready.
triggers:
  - create a PR
  - open a pull request
  - push and watch checks
  - watch PR checks
  - submit PR
  - monitor PR
---

# Pull Request Lifecycle

Complete workflow for submitting a PR, monitoring CI, and handling Codex
review comments. Follow every step — do not skip the Codex comment loop.

## 1. Create the PR

```bash
git push -u origin <branch>
gh pr create --title "<type>: <description>" --body "<summary>"
```

Keep the PR description short: what changed, why, and what validation ran.

## 2. Watch CI checks

Poll until all checks resolve (pass or fail):

```bash
gh pr checks <number>
```

- Poll every ~30s initially, back off to ~60s.
- If a check fails, investigate the failure, fix in code, commit, and push.
  Each push restarts the cycle from this step.

## 3. Poll for Codex review comments

After checks pass (or while waiting), poll for Codex review comments.
Codex reviews arrive asynchronously — typically within 1–3 minutes of a push.

```bash
gh api repos/{owner}/{repo}/pulls/{number}/comments \
  --jq '.[] | select(.user.login == "chatgpt-codex-connector[bot]") | {id, path, line, body}'
```

Or use the GraphQL query to get thread IDs (needed for resolving):

```bash
gh api graphql -f query='
{
  repository(owner: "{owner}", name: "{repo}") {
    pullRequest(number: {number}) {
      reviewThreads(first: 20) {
        nodes {
          id
          isResolved
          comments(first: 5) {
            nodes { body, author { login }, databaseId }
          }
        }
      }
    }
  }
}'
```

**Codex always leaves at least one comment per review** — either inline
feedback or a top-level "looks good" message. If you see **zero** Codex
comments after a push, the review is still in progress. Keep polling
(~30s intervals) until at least one Codex comment appears for the current
review cycle. Only after a Codex comment confirms the review is complete
(and there are no unresolved threads) can you move on.

## 4. Fix Codex comments

For each unresolved Codex comment:

1. **Fix the issue in code** — make the requested change.
2. **Commit and push** — use a descriptive commit message.
3. **Reply to the comment** explaining the fix:
   ```bash
   gh api repos/{owner}/{repo}/pulls/{number}/comments/{comment_id}/replies \
     -f body="Fixed: <explanation>"
   ```
4. **Resolve the review thread** via GraphQL:
   ```bash
   gh api graphql -f query='
     mutation {
       resolveReviewThread(input: {threadId: "{thread_id}"}) {
         thread { isResolved }
       }
     }'
   ```

## 5. Repeat

Each push may trigger a new Codex review. After pushing fixes:

1. Go back to **Step 2** — wait for CI checks to pass.
2. Go back to **Step 3** — poll for new Codex comments (~2 min).
3. If new comments appear, go to **Step 4**.

## 6. Declare ready

A PR is ready to merge **only when ALL of these are true**:

- [ ] All CI checks pass (green)
- [ ] Codex has posted at least one comment for the latest push (confirming the review completed)
- [ ] No unresolved Codex review threads
- [ ] No new unresolved comments after the latest push

Zero Codex comments ≠ "no issues". It means the review hasn't finished.
Only declare the PR ready after Codex has spoken.

## Common pitfalls

- **Don't skip the Codex poll** — `gh pr checks` passing does NOT mean
  there are no review comments. Always check both.
- **Don't resolve without fixing** — resolve threads only after the
  underlying issue is addressed in code and pushed.
- **Each push resets the clock** — new pushes can trigger new reviews.
  Always re-poll after pushing.
- **Zero comments ≠ all clear** — Codex always posts at least one comment
  per review. If you see none, the review is still running. Keep polling.
