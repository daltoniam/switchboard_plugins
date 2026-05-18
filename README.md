# switchboard_plugins

Third-party [Switchboard](https://github.com/daltoniam/switchboard) WASM plugins by [@daltoniam](https://github.com/daltoniam).

A home for integrations that make more sense as standalone WASM modules than as core Switchboard adapters.

## Plugins

| Plugin | Tools | Description |
|--------|-------|-------------|
| [bland](plugins/bland/) | 30 | Bland.ai voice AI: calls, transcripts, voices, pathways, inbound numbers, knowledge bases, org management, billing, audit logs |
| [looker](plugins/looker/) | 23 | Looker BI: dashboards, Looks, LookML models, inline analytics queries, SQL Runner |

Prebuilt binaries live in [`dist/`](dist/) and are referenced by [`manifest.json`](manifest.json).

## Install

In the Switchboard web UI, go to **Plugin Marketplace** and add this manifest URL:

```
https://raw.githubusercontent.com/daltoniam/switchboard_plugins/main/manifest.json
```

Or install an individual plugin by its `.wasm` URL.

## Build

Requires Rust with the `wasm32-wasip1` target:

```bash
rustup target add wasm32-wasip1
cargo build --release --target wasm32-wasip1
```

Outputs land at `target/wasm32-wasip1/release/<crate>_wasm.wasm`. To refresh committed binaries:

```bash
cp target/wasm32-wasip1/release/<plugin>_wasm.wasm dist/<plugin>.wasm
```

After updating a binary, regenerate its entry in [`manifest.json`](manifest.json) (bump `version`, update `sha256`, `size`, `released_at`).

## Plugin ABI

Each plugin is a `cdylib` exporting the standard Switchboard guest ABI:

| Export | Purpose |
|--------|---------|
| `name` | Plugin name |
| `metadata` | Version, ABI range, credential keys, capabilities |
| `tools` | Tool definitions returned to Switchboard |
| `configure(ptr_size)` | Receives credentials JSON |
| `execute(ptr_size)` | Runs a tool call |
| `healthy` | Liveness check (1 = healthy) |

Shared types come from [`switchboard-guest-sdk`](https://github.com/daltoniam/switchboard/tree/main/wasm/guest-rust/sdk), pulled via Cargo git dependency.

## Layout

```
.
├── Cargo.toml         # workspace = ["plugins/*"]
├── manifest.json      # Switchboard marketplace manifest
├── dist/              # Committed prebuilt WASM binaries
└── plugins/           # Plugin crates (one per WASM module)
```

## License

MIT
