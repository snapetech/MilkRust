# Source Import Audit

RustyMilk has two local source histories worth preserving without copying application code wholesale:

- `../slskdn`: JavaScript native MilkDrop/MilkDrop3 implementation.
- `../slskR`: later Rust/WASM RustyMilk integration and slskR host UI.

The JavaScript port is preserved in-tree at:

```text
archive/slskdn-js-milkdrop-port/
```

That archive is reference material, not the production RustyMilk engine.

## Imported Now

The first import pass brought the curated `../slskdn` MilkDrop fixture behavior into Rust core tests.

Source reference:

```text
../slskdn/src/web/src/components/Player/visualizers/milkdrop/presetFixtures.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/presetFixtures.test.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/presetCompatibilityMatrix.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/presetParser.test.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/presetCompatibility.test.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/expressionVm.test.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/shaderTranslator.test.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/milkdropRenderer.test.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/webgpuRenderer.test.js
../slskR/web/src/components/Player/visualizers/rustyMilkEngine.test.js
```

RustyMilk coverage added:

- Classic primitive fixture: per-frame, per-pixel, motion vectors, shape, sprite, and custom wave.
- Shader subset fixture: supported warp and comp shader body forms.
- `.milk2` double preset fixture.
- MilkDrop3 q-register fixture covering `q1`, `q2`, `q16`, `q32`, `q48`, `q63`, and `q64`.
- Dense primitive fixture with 40 shapes and 20 waves.
- Unsupported shader control-flow fixture with clean `comp_shader` compatibility reporting.
- Compatibility matrix summary assertions for support counts, preset counts, max primitive counts, q-register coverage, and unsupported shader sections.
- Expression VM parity for arithmetic, q-register assignments, compound assignment, comparisons, condition helpers, audio helpers, constants, bitwise helpers, shifts, unary operators, and logical operators.
- Shader translator parity for ret assignments, shader bodies, q/audio uniforms, FFT/waveform helpers, named texture samplers, GLSL output, WGSL output, and rejected unsafe shader bodies.
- Web SDK wrapper tests for transition helpers, beat detection, audio analyzer data flow, mouse state, resize, preset load/edit/export methods, debug summaries, and disposal across the Rust WASM boundary.
- Parser parity for comments, base values, per-frame/per-pixel equations, shader sections, indexed shapes/sprites/waves, MilkDrop3 `.milk2` double presets, primitive aliases, standalone fragments, prefixed fragments, serialization, and compatibility diagnostics.
- Renderer-neutral geometry and WebGPU packing parity for waveform vertices, motion vectors, screen borders, shape fills/outlines, sprites, UVs, colored vertex packing, textured vertex packing, and line/triangle conversion.

Notable behavior difference:

- The old JavaScript WebGPU translator rejected a ternary ret expression. RustyMilk currently accepts that safe case, so the Rust test preserves the newer Rust behavior instead of reintroducing the older limitation.
- The old JavaScript compatibility test treated `megabuf` as unsupported. RustyMilk currently supports `megabuf`/`gmegabuf` buffer state, so imported compatibility assertions preserve that newer Rust behavior.

Test location:

```text
crates/rustymilk-core/src/lib.rs
packages/rustymilk-web/src/rustyMilkEngine.test.js
```

The remaining browser/GPU/player import surface is mapped in [`RENDERER_AND_PLAYER_IMPORT_PLAN.md`](RENDERER_AND_PLAYER_IMPORT_PLAN.md).

## Still To Mine From `../slskdn`

These should be imported as behavior, fixtures, or requirements, not as app-specific source drops.

- `expressionVm.js` and tests: continue translating any remaining edge cases into Rust expression VM tests.
- `shaderTranslator.js` and tests: continue translating any remaining GLSL/WGSL safe-subset expectations into core and renderer tests.
- `milkdropRenderer.js` and tests: split useful WebGL2 behavior into future renderer backend tests.
- `webgpuRenderer.js` and tests: use as design/reference material for `rustymilk-renderer-wgpu`.
- `nativeMilkdropEngine.js` and tests: preserve SDK/player workflows for preset library, favorites, search, playlists, fragment import/export, automation, debug snapshots, FPS caps, quality presets, and texture assets.
- `smoke-native-milkdrop.mjs`: port context-loss and nonblank-canvas smoke coverage against RustyMilk packages.
- `measure-native-milkdrop-performance.mjs`: port fixture-based browser performance measurement.
- `report-native-milkdrop-compatibility.mjs`: replace app-path imports with RustyMilk CLI/package calls.
- `docs/design/webgl-milkdrop3-port.md`: fold remaining design notes into architecture docs and issues.

## Still To Check From `../slskR`

Use `../slskR` mostly as a parity source.

- Confirm every slskR `RustyMilkEngine` call exists in `crates/rustymilk-wasm`.
- Keep slskR smoke, compatibility, and performance expectations working against this standalone package.
- Extract player workflow requirements into RustyMilk Web SDK, Player, and Studio docs.
- Do not import slskR routing, static shell HTML, Soulseek-specific components, or app storage keys.

## Import Rules

- Canonical engine behavior belongs in Rust.
- JavaScript-era code is a compatibility reference, not the long-term runtime.
- Preset fixtures must be license-reviewed before becoming public sample packs.
- Renderer-specific behavior should wait for renderer backend crates unless it is already renderer-neutral.
- Host UI workflows should become SDK/Player/Studio requirements, not copied React components.
