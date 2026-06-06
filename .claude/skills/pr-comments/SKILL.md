---
name: pr-comments
description: Submit a PR review as inline GitHub comments on specific files and lines using the gh CLI.
---

# PR Comments

Post review findings as inline comments on specific diff lines via the GitHub Pull Request Reviews API and `gh` CLI.

## Workflow

1. Get the PR head SHA: `gh api repos/daltoniam/switchboard_plugins/pulls/<number> --jq '.head.sha'`.
2. Get the diff: `gh pr diff <number> --repo daltoniam/switchboard_plugins`.
3. Check existing comments: `gh api repos/daltoniam/switchboard_plugins/pulls/<number>/comments`.
4. Only comment on concrete, actionable findings. If there are no Must Fix or Should Fix findings, submit an approval with no inline comments.
5. Build the review payload with Python and submit it with:

```bash
gh api repos/daltoniam/switchboard_plugins/pulls/<number>/reviews \
  --method POST \
  --input /tmp/review_payload.json \
  --jq '.html_url'
```

Use a concise, human tone. Do not post placeholder comments.
