# AGENTS.md

## Overview

Workspace of Rust crates that compile to Switchboard WASM plugins. Each plugin is an independent `cdylib` exporting the Switchboard guest ABI.

## Commands

| Target | Command |
|--------|---------|
| Build all | `cargo build --release --target wasm32-wasip1` |
| Build one | `cargo build --release --target wasm32-wasip1 -p <plugin>-wasm` |
| Format | `cargo fmt` |
| Clippy | `cargo clippy --target wasm32-wasip1 -- -D warnings` |

## Adding a plugin

1. Scaffold `plugins/<name>/` with `Cargo.toml` (`crate-type = ["cdylib"]`) and `src/lib.rs`.
2. Depend on `switchboard-guest-sdk` via git: `{ git = "https://github.com/daltoniam/switchboard.git" }`.
3. Export the 6 required ABI functions (`name`, `metadata`, `tools`, `configure`, `execute`, `healthy`) — the SDK provides helpers for all of them.
4. Build, copy output to `dist/<name>.wasm`, and add an entry to `manifest.json` with `sha256`, `size`, `released_at`, and a download URL pointing at `dist/<name>.wasm` on `main`.

## Conventions

- Crate name: `<plugin>-wasm` so the artifact is `<plugin>_wasm.wasm`.
- Tool naming: `<plugin>_<verb>_<noun>` (matches core Switchboard convention).
- No panics in tool handlers — return structured errors via the SDK.
- Configuration is stored in a static `Mutex` populated by `configure`.
- Optimize for size: `opt-level = "z"`, `lto = true`, `strip = true`, `panic = "abort"` (set in the workspace root `Cargo.toml`).

## Manifest format

See [Switchboard plugin marketplace docs](https://github.com/daltoniam/switchboard) for the authoritative schema. Each plugin has one or more `versions` entries; bump the version on every binary change.
