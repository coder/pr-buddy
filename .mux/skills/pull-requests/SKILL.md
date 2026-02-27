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

Codex may leave **inline thread comments**, **top-level review comments**,
or both. You must check both sources:

**Check inline thread comments** (also gives thread IDs for resolving):

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

**Check top-level PR reviews** (catches summary-only reviews with no inline comments).
Include `commit_id` to scope to the latest push — old reviews from earlier
commits don't count:

```bash
gh api repos/{owner}/{repo}/pulls/{number}/reviews \
  --jq '.[] | select(.user.login == "chatgpt-codex-connector[bot]") | {id, state, commit_id, body}'
```

Compare `commit_id` against the current HEAD (`git rev-parse HEAD`). Only
reviews matching the latest commit confirm that Codex has reviewed it.

**Codex always leaves at least one review or comment per review cycle.**
If you see zero Codex activity **for the latest commit** from both queries,
the review is still in progress. Keep polling (~30s intervals) until at
least one Codex review or comment appears for the current HEAD. Only after
Codex has spoken (and there are no unresolved threads) can you move on.

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
- [ ] Codex has posted at least one review or comment for the latest push (check both reviews and review threads)
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
- **Zero comments ≠ all clear** — Codex always posts at least one review
  or comment per review cycle. Check both `/reviews` and `reviewThreads`.
  If both are empty for the current cycle, the review is still running.
  Keep polling.
