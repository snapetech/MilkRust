# RustyMilk

[![CI](https://github.com/snapetech/RustyMilk/actions/workflows/ci.yml/badge.svg)](https://github.com/snapetech/RustyMilk/actions/workflows/ci.yml)

RustyMilk is the standalone Rust/WASM visualizer engine extracted from slskR.

This repository is being prepared as the home for:

- `rustymilk-core`: preset parsing, expression evaluation, runtime frame generation, compatibility reports, geometry, and WebGPU batch builders.
- `rustymilk-pack`: portable preset pack manifests, loading, and validation.
- `rustymilk-renderer-core`: renderer backend contracts and capability types.
- `rustymilk-renderer-headless`: headless renderer stats backend for tests, reports, and batch tooling.
- `rustymilk-cli`: command-line validation, inspection, compatibility, and render-stat tooling.
- `rustymilk-wasm`: browser-facing WASM bindings and renderers.
- `packages/rustymilk-web`: JavaScript client wrapper for applications that consume the WASM package.
- `apps/rustymilk-player`: standalone browser player prototype.
- `apps/rustymilk-studio`: browser authoring/debugging prototype.
- `tools`: smoke, compatibility, and performance checks.
- `examples`: small browser clients used for SDK verification.

The current migration keeps legacy `.milk` and `.milk2` preset formats compatible while removing product naming from the engine identity.

See [`docs/ROADMAP.md`](docs/ROADMAP.md) for the expanded plan covering the core engine, renderer backends, web SDK, CLI, preset packs, plugins, standalone player, Studio tooling, language SDKs, and host integrations. See [`docs/PRESET_PACKS.md`](docs/PRESET_PACKS.md), [`docs/THIRD_PARTY_CONTENT_POLICY.md`](docs/THIRD_PARTY_CONTENT_POLICY.md), [`docs/THIRD_PARTY_CONTENT_AUDIT.generated.md`](docs/THIRD_PARTY_CONTENT_AUDIT.generated.md), [`docs/SOURCE_IMPORT_AUDIT.md`](docs/SOURCE_IMPORT_AUDIT.md), [`docs/RENDERER_AND_PLAYER_IMPORT_PLAN.md`](docs/RENDERER_AND_PLAYER_IMPORT_PLAN.md), and [`archive/slskdn-js-milkdrop-port`](archive/slskdn-js-milkdrop-port) for the current pack format, content policy, preserved slskdN JavaScript port, and import checklist.

## Build

```bash
cargo test --workspace
cargo check -p rustymilk-wasm --target wasm32-unknown-unknown
npm run build:wasm
```

`npm run build:wasm` requires the `wasm-bindgen` CLI and writes browser package files to `pkg/`.

## Apps

```bash
npm run build:wasm
npm run dev:player
npm run dev:studio
```

The app commands serve the local repo at `http://127.0.0.1:4173/`.

## CLI

```bash
cargo run -p rustymilk-cli -- validate preset.milk
cargo run -p rustymilk-cli -- inspect preset.milk
cargo run -p rustymilk-cli -- compat ./presets
cargo run -p rustymilk-cli -- render-stats preset.milk
cargo run -p rustymilk-cli -- pack-inspect examples/sample-pack
cargo run -p rustymilk-cli -- pack-validate examples/sample-pack
```

## Content Catalog

```bash
npm run content:audit
npm run content:validate
```

Third-party packs live in [`content/catalog.json`](content/catalog.json). Vendored content requires explicit redistribution permission; unclear historical MilkDrop packs are tracked as link-only until their license is reviewed. The repo currently vendors the MIT-licensed [`butterchurn-presets@2.4.7`](content/third-party/butterchurn-presets-2.4.7) converted preset package.
