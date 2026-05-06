# Architecture

RustyMilk is split into layers so hosts can use only what they need.

- Core: pure Rust parser, expression VM, preset documents, compatibility analysis, frame runtime, primitive geometry, and packed renderer batches.
- WASM: `wasm-bindgen` exports, WebGL2/canvas renderers, browser texture plumbing, and the public `RustyMilkEngine` class.
- Web client: JavaScript convenience wrapper that connects Web Audio analyzers, preset libraries, automation, and host UI callbacks.
- Tools: headless smoke, compatibility, and performance checks that should work against either a built WASM package or an embedding app.

The extraction target is for slskR to depend on RustyMilk instead of carrying the engine implementation inline.

