# Architecture

RustyMilk is split into layers so hosts can use only what they need.

- Core: pure Rust parser, expression VM, preset documents, compatibility analysis, frame runtime, primitive geometry, and packed renderer batches.
- Pack: pure Rust folder-pack manifest parsing, path-safety checks, preset compatibility validation, and shared metadata for SDK/player/Studio imports.
- Renderer core: shared renderer traits, capability reports, and render statistics contracts.
- Headless renderer: renderer-neutral test/report backend that consumes frame sets or packed WebGPU-style batches.
- CLI: validation, inspection, compatibility, and headless render-stat tooling built on core plus renderer contracts.
- WASM: `wasm-bindgen` exports, WebGL2/canvas renderers, browser texture plumbing, and the public `RustyMilkEngine` class.
- Web client: JavaScript convenience wrapper that connects Web Audio analyzers, preset libraries, automation, and host UI callbacks.
- Plugin APIs: JSON/JS plugin descriptors loaded from pack manifests and executed by the web client lifecycle hooks; native Rust hosts also expose `DesktopPlugin` hook points through `rustymilk-desktop`.
- Player app: standalone browser player prototype using the web SDK.
- Studio app: browser authoring/debugging prototype using the WASM engine directly.
- Tools: headless smoke, compatibility, and performance checks that should work against either a built WASM package or an embedding app.
- Desktop host primitives: `rustymilk-desktop` crate with deterministic frame/runtime session
  primitives, shared preset source resolution utilities, a reusable `DesktopPlayerEngine` playback API
  with pluggable `DesktopAudioProvider` input, and split desktop binaries (`probe`, `player`, `studio`)
  plus an optional `player-ui` native shell behind the `ui` feature flag for playback and compatibility workflows.

The extraction target is for slskR to depend on RustyMilk instead of carrying the engine implementation inline.
