# RustyMilk

[![CI](https://github.com/snapetech/RustyMilk/actions/workflows/ci.yml/badge.svg)](https://github.com/snapetech/RustyMilk/actions/workflows/ci.yml)

RustyMilk is the standalone Rust/WASM visualizer engine extracted from slskR.

This repository is being prepared as the home for:

- `rustymilk-core`: preset parsing, expression evaluation, runtime frame generation, compatibility reports, geometry, and WebGPU batch builders.
- `rustymilk-wasm`: browser-facing WASM bindings and renderers.
- `packages/rustymilk-web`: JavaScript client wrapper for applications that consume the WASM package.
- `tools`: smoke, compatibility, and performance checks.
- `examples`: small browser clients used for SDK verification.

The current migration keeps legacy `.milk` and `.milk2` preset formats compatible while removing product naming from the engine identity.

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for the expanded plan covering the core engine, renderer backends, web SDK, CLI, preset packs, plugins, standalone player, Studio tooling, language SDKs, and host integrations. See [`docs/SOURCE_IMPORT_AUDIT.md`](docs/SOURCE_IMPORT_AUDIT.md) for the slskdN/slskR import checklist.

## Build

```bash
cargo test --workspace
cargo check -p rustymilk-wasm --target wasm32-unknown-unknown
npm run build:wasm
```

`npm run build:wasm` requires the `wasm-bindgen` CLI and writes browser package files to `pkg/`.
