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

Codex reviews are triggered automatically when a PR is opened or marked
ready, but **not** on subsequent pushes. After pushing fixes, you must
explicitly request a new review by commenting on the PR:

```bash
gh pr comment {number} --body "@codex review"
```

After requesting, poll for Codex review comments. Codex reviews arrive
asynchronously — typically within 1–3 minutes.

**Preferred: use the helper script** which checks all Codex response
channels (👍 reactions, PR comments, and review threads):

```bash
./scripts/wait_pr_codex.sh {number}        # blocks until Codex responds
./scripts/check_codex_comments.sh {number} # one-shot check (exit 0/1/10)
```

Exit codes: `0` = approved, `1` = comments to address, `10` = still waiting.

**Note:** The script can return `0` from a stale 👍 reaction left on a
prior commit. If you just pushed and requested `@codex review`, also
check for new unresolved threads or comments (by timestamp) to confirm
the new review cycle has actually completed.

**Manual polling** (when scripts aren't available): Codex may respond via
👍 reactions on the PR, regular PR comments, or inline review threads.
You must check all three — the GraphQL query below covers them in one call:

```bash
gh api graphql -f query='
{
  repository(owner: "{owner}", name: "{repo}") {
    pullRequest(number: {number}) {
      comments(last: 20) {
        nodes { author { login }, body, createdAt }
      }
      reviewThreads(last: 20) {
        nodes {
          id
          isResolved
          comments(first: 1) {
            nodes { author { login }, body, databaseId }
          }
        }
      }
      reactions(last: 20, content: THUMBS_UP) {
        nodes { user { login } }
      }
    }
  }
}'
```

Filter comments/threads for `author.login == "chatgpt-codex-connector"`
and reactions for `user.login == "chatgpt-codex-connector"` (reactions use
`user`, not `author`).

**⚠️ Stale reactions:** A 👍 reaction from a prior review cycle persists
across pushes. After requesting a new `@codex review`, don't treat an
existing reaction as proof the new review is complete. Look for a **new**
comment or unresolved thread posted after your `@codex review` request
(compare `createdAt` timestamps).

**Codex always leaves at least one response per review cycle** (reaction,
comment, or review thread). If you see zero **new** Codex activity after
requesting a review, it is still in progress. Keep polling (~30s intervals)
until Codex has spoken. Only then check for unresolved threads.

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

Codex does **not** automatically re-review after a push. After pushing fixes:

1. Go back to **Step 2** — wait for CI checks to pass.
2. **Request a new Codex review**: `gh pr comment {number} --body "@codex review"`
3. Go back to **Step 3** — poll for new Codex comments.
4. If new comments appear, go to **Step 4**.

## 6. Declare ready

A PR is ready to merge **only when ALL of these are true**:

- [ ] All CI checks pass (green)
- [ ] Codex has responded for the latest review cycle (reaction, comment, or review thread)
- [ ] No unresolved Codex review threads
- [ ] No new unresolved comments after the latest push

Zero Codex comments ≠ "no issues". It means the review hasn't finished.
Only declare the PR ready after Codex has spoken.

## Common pitfalls

- **Don't skip the Codex poll** — `gh pr checks` passing does NOT mean
  there are no review comments. Always check both.
- **Don't resolve without fixing** — resolve threads only after the
  underlying issue is addressed in code and pushed.
- **Codex doesn't auto-review pushes** — after pushing fixes, you must
  comment `@codex review` to trigger a new review. Always do this and
  re-poll after pushing.
- **Zero activity ≠ all clear** — Codex always responds per review cycle
  (reaction, comment, or thread). Check all three channels. If all are
  empty, the review is still running. Keep polling.
