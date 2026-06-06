---
name: pr-review
description: Review a GitHub pull request for Switchboard WASM plugins.
---

# Pull Request Review

Review the PR for correctness, security, maintainability, test coverage, and production readiness. Be direct, concise, and actionable.

## Workflow

1. Fetch PR context with `gh pr view <number> --repo daltoniam/switchboard_plugins --json title,body,headRefName,baseRefName,state,author,files,additions,deletions,changedFiles`.
2. Fetch the full diff with `gh pr diff <number> --repo daltoniam/switchboard_plugins`.
3. Check existing review comments with `gh api repos/daltoniam/switchboard_plugins/pulls/<number>/comments` and avoid duplicates.
4. Read `AGENTS.md` for project conventions.
5. Verify changed code builds and formats when feasible:
   - `cargo fmt --check`
   - `cargo clippy --target wasm32-wasip1 -- -D warnings`
   - `cargo build --release --target wasm32-wasip1`
6. Review for:
   - Switchboard guest ABI correctness
   - No panics in tool handlers
   - Safe configuration handling
   - Manifest/version consistency for binary changes
   - Rust idioms and error handling
   - Tests or validation for changed behavior
7. Produce a structured review with Must Fix, Should Fix, Consider, Verification, and Questions sections.

Only raise findings that are concrete and actionable. Do not nitpick formatting handled by tooling.
