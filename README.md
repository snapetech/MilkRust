# RustyMilk

RustyMilk is the standalone Rust/WASM visualizer engine extracted from slskR.

This repository is being prepared as the home for:

- `rustymilk-core`: preset parsing, expression evaluation, runtime frame generation, compatibility reports, geometry, and WebGPU batch builders.
- `rustymilk-wasm`: browser-facing WASM bindings and renderers.
- `packages/rustymilk-web`: JavaScript client wrapper for applications that consume the WASM package.
- `tools`: smoke, compatibility, and performance checks.
- `examples`: small browser clients used for SDK verification.

The current migration keeps legacy `.milk` and `.milk2` preset formats compatible while removing product naming from the engine identity.

## Build

```bash
cargo test --workspace
cargo check -p rustymilk-wasm --target wasm32-unknown-unknown
npm run build:wasm
```

`npm run build:wasm` requires the `wasm-bindgen` CLI and writes browser package files to `pkg/`.
