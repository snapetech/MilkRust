# Renderer And Player Import Plan

This plan captures the remaining `../slskdn` JavaScript MilkDrop import surface that should not be copied directly into RustyMilk yet. The durable behavior should land in renderer crates, smoke tools, SDK tests, Desktop Player requirements, or Studio requirements as those surfaces appear.

## Already Imported Into Core

RustyMilk core now covers the renderer-neutral pieces from the old WebGL/WebGPU tests:

- Waveform vertex placement for classic modes.
- Motion vector geometry.
- Inner and outer screen border geometry.
- Shape outline and fill geometry.
- Sprite quad geometry and UVs.
- WebGPU colored triangle-list packing.
- WebGPU triangle-fan packing.
- WebGPU textured triangle-fan packing.
- WebGPU line-list and line-strip segment packing.
- WebGPU motion-vector packing.
- WebGPU screen-border packing.
- WebGPU shape fill, textured shape, sprite, textured sprite, and outline packing.

Source references:

```text
../slskdn/src/web/src/components/Player/visualizers/milkdrop/milkdropRenderer.test.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/webgpuRenderer.test.js
```

RustyMilk test location:

```text
crates/rustymilk-core/src/lib.rs
```

## Future Renderer Backend Tests

When renderer crates are split, import these as backend tests instead of core tests.

### WebGL2

Target crate: `rustymilk-renderer-webgl`

Import coverage:

- WebGL2 unavailable error path.
- GPU-backed frame color draw.
- Feedback ping-pong texture allocation and resize.
- Preset decay as feedback blend.
- WebGL resource disposal.
- Shader compile/link failure reporting.
- Translated warp and comp shader render passes.
- Per-pixel warp grid rendering.
- Texture asset upload and alias lookup by path, basename, and stem.
- Procedural texture fallback.
- Textured shape rendering.
- Sprite rendering.
- Additive custom shape blending.
- Final composite blend modes.
- WebGL attribute rebinding after program switches.
- Context-loss and restore smoke coverage.

### WebGPU/wgpu

Target crate: `rustymilk-renderer-wgpu`

Import coverage:

- WebGPU unavailable status without throwing.
- Adapter capability probing without forced device creation.
- Device/canvas pipeline creation.
- Feedback texture setup.
- Canvas display pass.
- Primitive draw buffers for waveform, shapes, sprites, motion vectors, and borders.
- Texture upload and named sampler binding.
- Safe WGSL shader execution.
- Shader-side FFT/waveform uniforms.
- WebGPU readiness reporting separate from WebGL compatibility.

## Smoke And Performance Tools

Source references:

```text
../slskdn/src/web/scripts/smoke-native-milkdrop.mjs
../slskdn/src/web/scripts/measure-native-milkdrop-performance.mjs
../slskdn/src/web/scripts/report-native-milkdrop-compatibility.mjs
```

RustyMilk targets:

- `tools/smoke-rustymilk.mjs`
- `tools/measure-rustymilk-performance.mjs`
- `tools/report-rustymilk-compatibility.mjs`
- Future `rustymilk` CLI commands.

Import coverage:

- Render curated fixtures in Chromium and fail on blank canvas pixel stats.
- Verify WebGL context loss and restore when the browser extension is available.
- Measure per-fixture frame timings.
- Accept local `.milk` / `.milk2` files and folders.
- Report skipped unsupported presets without aborting whole batches.
- Emit human-readable and JSON summaries.

## Web SDK And Player Workflow Requirements

Source references:

```text
../slskdn/src/web/src/components/Player/visualizers/nativeMilkdropEngine.test.js
../slskdn/src/web/src/components/Player/Visualizer.test.jsx
```

These should become SDK, Desktop Player, or Studio behavior rather than copied React app code.

Import coverage:

- Engine selection between WebGL2 and WebGPU with WebGL fallback.
- Timed and beat-based automation.
- Preset transitions: crossfade, cut, fade, overlay.
- `.milk2` double preset rendering and secondary composite modes.
- Preset-defined transition duration and composite aliases.
- Import-time compatibility checks before replacing the active renderer.
- Rejection of unsupported secondary `.milk2` presets.
- `.shape` and `.wave` fragment import/export/removal.
- Parameter editing and randomized editable parameters.
- Debug snapshots with format, renderer, shader sections, primitive counts, and WebGPU status.
- Pointer/mouse variables.
- Texture asset import and scoped asset persistence.
- Folder import using browser relative paths.
- Texture size/type skip reporting.
- Browser-local preset library.
- Favorites, history, previous, next, random.
- Search/filter scoped navigation.
- Playlist save, activate, clear, delete, and rename.
- Render error surfacing and bad-import cleanup.

## Studio Requirements

RustyMilk Studio should reuse the same imported workflows:

- Preset inspector.
- Compatibility report UI.
- Fragment editor/import/export.
- Parameter editor and randomized mashups.
- Texture manager with alias diagnostics.
- Playlist and pack builder.
- Shader translator/debug panel.
- Renderer debug panel.
- Thumbnail/smoke preview for every preset in a pack.

## Do Not Import

- slskdN React component structure.
- slskdN local-storage key names.
- Butterchurn-specific host code.
- Soulseek/slskdN application shell concerns.
- Vite app path assumptions.
- Browser-only renderer code into `rustymilk-core`.

