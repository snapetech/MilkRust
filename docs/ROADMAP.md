# Roadmap

RustyMilk should grow from an extracted Rust/WASM visualizer engine into a portable MilkDrop-compatible ecosystem: a reusable core engine, renderer backends, SDKs, standalone clients, authoring tools, plugin surfaces, preset packs, and automation tooling.

This roadmap is organized around product surfaces and build phases. The near-term priority is still clean extraction from slskR, but every extraction step should move the project toward stable APIs and multi-host reuse.

## Product Frame

RustyMilk should eventually ship as these related products:

- **RustyMilk Core**: pure Rust parser, preset document model, expression VM, runtime frame generation, compatibility analysis, and renderer-neutral frame output.
- **RustyMilk Renderers**: modular render backends for WebGL2, canvas fallback, WebGPU/wgpu, native GPU windows, and headless capture.
- **RustyMilk Web SDK**: installable TypeScript/JavaScript package for browser, Electron, Tauri webviews, and web apps.
- **RustyMilk Desktop Player**: standalone MilkDrop-style visualizer client with fullscreen playback, preset browsing, automation, audio input selection, and capture/export tools.
- **RustyMilk Studio**: preset authoring, debugging, compatibility inspection, fragment editing, texture management, and pack publishing.
- **RustyMilk CLI**: command-line validation, inspection, compatibility reports, thumbnails, offline renders, conversion, packing, and benchmarking.
- **RustyMilk Packs**: distributable preset, texture, fragment, metadata, and plugin bundles.
- **RustyMilk Plugins**: extension points for preset packs, audio analyzers, beat detectors, automation, input devices, post-processing, exports, and host integrations.

## Current Baseline

The repository currently contains:

- `crates/rustymilk-core`: parser/runtime-facing Rust code, frame structs, preset handling, compatibility helpers, geometry, and render batch summaries.
- `crates/rustymilk-pack`: preset pack manifest parser, folder-pack loader, path-safety validation, and preset compatibility reporting.
- `crates/rustymilk-renderer-core`: renderer contracts, capabilities, and render statistics.
- `crates/rustymilk-renderer-headless`: headless renderer stats backend for tests and tooling.
- `crates/rustymilk-cli`: validation, inspection, compatibility, and headless render-stat commands.
- `crates/rustymilk-wasm`: `wasm-bindgen` exports, browser-facing `RustyMilkEngine`, and WebGL2/canvas rendering.
- `packages/rustymilk-web`: JavaScript convenience wrapper for Web Audio, automation, preset loading, and WASM consumption.
- `apps/rustymilk-player`: standalone browser player prototype.
- `apps/rustymilk-studio`: browser authoring/debugging prototype.
- `tools`: smoke, compatibility, and performance checks.
- `examples`: browser smoke client.
- `content/catalog.json`: third-party content catalog with copy/link/review policy.
- `content/third-party/butterchurn-presets-2.4.7`: MIT-licensed converted Butterchurn preset package vendored for compatibility/import work.
- `content/community-unlicensed`: public projectM/MilkDrop community preset and texture packs imported for aggressive compatibility testing and opt-in builds.
- `content/generated`: generated summaries for community pack counts and sampled compatibility.

The immediate migration target is for slskR to depend on RustyMilk instead of carrying the engine inline.

## Source Import Audit

RustyMilk has two important neighboring source histories:

- `../slskdn`: the earlier JavaScript native MilkDrop/MilkDrop3 implementation.
- `../slskR`: the later Rust/WASM RustyMilk integration and slskR player UI.

These should not be imported wholesale. They should be mined deliberately for missing behavior, fixtures, tests, docs, and host UI workflows.

The working checklist lives in [`SOURCE_IMPORT_AUDIT.md`](SOURCE_IMPORT_AUDIT.md).

### `../slskdn` JavaScript MilkDrop Source

This is the richer compatibility reference. It contains a full browser-native MilkDrop-era implementation under:

```text
../slskdn/src/web/src/components/Player/visualizers/nativeMilkdropEngine.js
../slskdn/src/web/src/components/Player/visualizers/milkdrop/
../slskdn/src/web/scripts/smoke-native-milkdrop.mjs
../slskdn/src/web/scripts/report-native-milkdrop-compatibility.mjs
../slskdn/src/web/scripts/measure-native-milkdrop-performance.mjs
../slskdn/docs/design/webgl-milkdrop3-port.md
```

Import candidates:

- Preset parser fixtures and edge cases.
- Expression VM behavior and tests, especially unsupported function diagnostics.
- Shader translator tests and safe-subset expectations.
- Compatibility matrix logic and metrics.
- Curated fixture pack covering classic primitives, textures, sprites, shader sections, `.milk2` double presets, q-registers, and dense primitive counts.
- WebGPU renderer concepts and tests.
- WebGL context-loss smoke coverage.
- Native MilkDrop design notes and historical phase checklist.
- Player workflows for preset library, favorites, search, playlists, fragment import/export, parameter editing, automation, debug snapshots, FPS caps, and quality presets.

Do not directly port the JavaScript renderer as the long-term engine. Use it as a behavioral reference while keeping the canonical implementation in Rust core plus modular renderer backends.

### `../slskR` Rust/WASM Source

This is mostly the extraction source and current host integration. It contains:

```text
../slskR/crates/slskr-web/src/lib.rs
../slskR/web/src/components/Player/visualizers/rustyMilkEngine.js
../slskR/web/src/components/Player/Visualizer.jsx
../slskR/web/src/components/Player/Visualizer.test.jsx
../slskR/web/scripts/smoke-rustymilk.mjs
../slskR/web/scripts/report-rustymilk-compatibility.mjs
../slskR/web/scripts/measure-rustymilk-performance.mjs
```

Most of the standalone web wrapper and scripts have already been copied here with product naming and package paths changed. `../slskR/crates/slskr-web/src/lib.rs` still has the old monolithic host crate shape, so it should be used for parity checks rather than copied back into RustyMilk.

Import candidates:

- Any RustyMilk-specific tests not already represented in this repo.
- Player UI behavior and tests that should become SDK examples or RustyMilk Studio/player requirements.
- slskR integration boundary docs.
- Any missing WASM export methods if slskR still calls APIs not present in `crates/rustymilk-wasm`.

Do not import slskR application shell, routing, static UI, or Soulseek-specific code into RustyMilk.

### Import Workstream

- Create a migration checklist from `../slskdn` MilkDrop modules to RustyMilk modules.
- Convert JS parser/VM/shader fixtures into Rust tests where the behavior belongs in `rustymilk-core`.
- Convert renderer-specific WebGL/WebGPU tests into renderer backend tests after renderer modularization.
- Move curated preset fixtures into this repo under a license-reviewed fixture location.
- Bring compatibility/performance scripts forward only when they operate against RustyMilk packages, not slskdN or slskR app paths.
- Preserve host UI workflows as requirements for the future Web SDK, Desktop Player, and Studio rather than embedding app-specific React components.
- Add parity checks that compare slskR host expectations against the standalone RustyMilk API until slskR fully consumes the package.

## Guiding Principles

- Keep `rustymilk-core` host-independent and deterministic.
- Treat renderer output as a stable contract instead of binding the core to one graphics API.
- Prefer additive modules and crates over a single monolithic package.
- Keep MilkDrop `.milk` and `.milk2` compatibility central.
- Make compatibility measurable with reports, fixtures, and reproducible tests.
- Design SDKs around real host workflows: browser apps, native desktop apps, authoring tools, plugins, automation, and batch processing.
- Stabilize low-level APIs before building large app surfaces on top of them.

## Target Repository Shape

The project does not need all of this immediately, but this is the intended direction.

```text
crates/
  rustymilk-core/
  rustymilk-expr/
  rustymilk-preset/
  rustymilk-runtime/
  rustymilk-renderer-core/
  rustymilk-renderer-webgl/
  rustymilk-renderer-wgpu/
  rustymilk-renderer-canvas/
  rustymilk-renderer-headless/
  rustymilk-audio/
  rustymilk-cli/
  rustymilk-desktop/

packages/
  rustymilk-web/
  rustymilk-react/
  rustymilk-node/
  rustymilk-authoring/
  rustymilk-presets-default/

apps/
  rustymilk-player/
  rustymilk-studio/

examples/
  browser-basic/
  react-basic/
  tauri-player/
  native-wgpu/
  node-thumbnailer/
```

## Phase 0: Extraction And Stabilization

Goal: finish the standalone migration without expanding the surface too quickly.

- Update slskR to consume `rustymilk-core`, `rustymilk-wasm`, and/or `packages/rustymilk-web`.
- Preserve parity with the old in-tree slskR implementation.
- Add migration fixtures for representative `.milk` and `.milk2` presets.
- Publish internal/private package builds before public releases.
- Convert `packages/rustymilk-web` into an installable package with typed exports.
- Delete the old in-tree slskR implementation after parity tests pass.
- Document the current API boundary between slskR and RustyMilk.

Exit criteria:

- slskR uses RustyMilk as an external dependency.
- Existing smoke, compatibility, and performance checks pass.
- README and package docs explain how to consume the standalone engine.

## Phase 1: Core Engine Foundation

Goal: make `rustymilk-core` the stable engine contract.

- Define stable Rust APIs for parsing, validation, inspection, serialization, and runtime rendering.
- Split or internally organize the core into preset parsing, expression evaluation, runtime state, compatibility analysis, geometry, and frame output.
- Add a typed preset document model for `.milk`, `.milk2`, preset sets, fragments, textures, and metadata.
- Add deterministic runtime options for tests, thumbnails, offline renders, and replay captures.
- Add preset normalization and migration helpers.
- Add structured diagnostics with error codes, warnings, source locations where possible, and remediation hints.
- Expand parser round-trip tests and expression VM fixtures.
- Add public API examples for Rust consumers.

Candidate modules or future crates:

- `rustymilk-preset`: preset documents, parsing, fragments, serialization, pack metadata.
- `rustymilk-expr`: expression parser, VM, scope handling, math compatibility.
- `rustymilk-runtime`: frame runtime, transitions, automation hooks, deterministic replay.

Exit criteria:

- Core APIs can be used without browser, WASM, canvas, or Web Audio dependencies.
- A host can parse a preset, inspect it, render frame data, and receive compatibility diagnostics using only Rust.

## Phase 2: Renderer Backend Architecture

Goal: make renderers replaceable and host-specific.

- Define `rustymilk-renderer-core` traits and data contracts.
- Move renderer-neutral types out of browser-specific code.
- Keep WebGL2 as the first production renderer.
- Keep canvas as a debug/fallback renderer.
- Add a headless renderer for thumbnails, tests, and batch output.
- Prototype WebGPU/wgpu for native and browser targets.
- Define renderer capability reporting: shaders, textures, warp mesh, feedback buffers, blend modes, maximum texture size, precision, and fallback reasons.
- Add golden-frame or perceptual snapshot tests for selected fixtures.
- Add performance budgets per renderer.

Candidate backends:

- `rustymilk-renderer-webgl`: browser WebGL2.
- `rustymilk-renderer-canvas`: 2D fallback and debug rendering.
- `rustymilk-renderer-wgpu`: native and browser WebGPU path.
- `rustymilk-renderer-headless`: image buffers, thumbnails, video frame sequences.
- `rustymilk-renderer-gl`: optional native OpenGL if wgpu is insufficient for a target.

Exit criteria:

- The runtime can produce frames without knowing which renderer consumes them.
- Browser and native renderers share the same frame contract.

## Phase 3: Web SDK

Goal: make the browser package feel like a real SDK, not a thin wrapper.

- Convert `packages/rustymilk-web` to TypeScript.
- Publish typed ESM exports.
- Provide high-level `createRustyMilkEngine()` for common usage.
- Provide lower-level APIs for advanced hosts: runtime, preset library, audio source, texture assets, automation, and diagnostics.
- Add React bindings in `packages/rustymilk-react`.
- Add a Web Component wrapper: `<rustymilk-visualizer>`.
- Add browser examples for vanilla JS, Vite, React, and CDN usage.
- Add support for user-provided audio sample streams, not only Web Audio analyzer nodes.
- Add asset loading helpers for textures and preset packs.
- Add SDK docs with lifecycle, resize, render loop, preset loading, automation, and cleanup examples.

Public API surfaces to design:

- Engine lifecycle: create, resize, render, dispose.
- Preset management: load, inspect, export, fragments, packs, playlists.
- Audio: Web Audio analyzer, custom samples, offline samples, silence/test sources.
- Automation: timed, beat-based, custom strategy callback.
- Input: mouse, touch, keyboard, gamepad, MIDI/OSC bridges.
- Diagnostics: renderer status, preset compatibility, performance counters.

Exit criteria:

- A web app can embed RustyMilk with typed APIs and no repo-specific assumptions.
- Browser examples double as SDK verification.

## Phase 4: CLI And Batch Tooling

Goal: make RustyMilk useful in scripts, CI, content pipelines, and compatibility work.

Add a `rustymilk` CLI with commands such as:

```text
rustymilk validate preset.milk
rustymilk inspect preset.milk --json
rustymilk compat presets/ --report report.html
rustymilk render preset.milk --audio song.wav --out frame.png
rustymilk thumbnail presets/*.milk --out thumbnails/
rustymilk convert old.milk --format milk2
rustymilk normalize preset.milk --out normalized.milk
rustymilk pack ./presets ./textures --out collection.rmpack
rustymilk bench presets/
```

Implementation work:

- Add `crates/rustymilk-cli`.
- Reuse core parser, compatibility analyzer, and headless renderer.
- Emit human-readable and JSON output.
- Add fixture-based CLI tests.
- Make compatibility and performance tools call the CLI where practical.

Exit criteria:

- Preset authors and CI jobs can validate, inspect, render, and report without writing code.

## Phase 5: Preset Pack Format

Goal: define a portable content bundle format.

Proposed extension: `.rmpack`

Proposed structure:

```text
manifest.json
presets/
textures/
fragments/
thumbnails/
plugins/
licenses/
```

Manifest fields:

- Pack name, ID, version, author, description.
- License and source URLs.
- Preset list with titles, tags, compatibility score, and thumbnail references.
- Texture aliases and usage.
- Required RustyMilk version.
- Optional plugin declarations.
- Optional playlist and automation defaults.

Work items:

- Define manifest schema.
- Add pack validation to core or CLI. Initial folder-pack validation now lives in `rustymilk-pack` and is exposed through `rustymilk pack-inspect` and `rustymilk pack-validate`.
- Add pack loading to web SDK and desktop player.
- Add pack publishing/export from Studio.
- Add default sample pack. `examples/sample-pack` is the initial local fixture pack.
- Maintain a third-party content catalog and audit generated from local/source scans.

Exit criteria:

- A host can load one file or folder and receive presets, textures, metadata, thumbnails, and optional extensions.

## Phase 6: Plugin Architecture

Goal: support extension without hardcoding every host feature into the engine.

Plugin categories:

- Preset packs and texture packs.
- Fragment packs: shapes, waves, shaders, snippets.
- Audio analyzers and beat detectors.
- Automation strategies.
- Input controllers: MIDI, OSC, keyboard, gamepad, mouse, touch.
- Post-process effects.
- Export targets: image, video, streaming, Spout, Syphon, NDI.
- Host integrations: OBS, VST/AU/LV2, desktop shells.

Initial approach:

- Start with data-only plugins through `.rmpack`.
- Add JavaScript plugin hooks in the web SDK.
- Add Rust trait-based plugins for native hosts.
- Consider WASI plugins later for sandboxed third-party logic.

Core plugin hooks to design:

- `onPresetLoaded`
- `onFrameStart`
- `onAudioFrame`
- `onBeat`
- `onInput`
- `onAutomationStep`
- `onRenderFrame`
- `onExport`

Exit criteria:

- Third-party packs and host-specific integrations can extend RustyMilk without modifying core engine code.

## Phase 7: Standalone Desktop Player

Goal: build a MilkDrop3-style RustyMilk client.

Implementation options:

- Fastest path: Tauri or Electron using the web SDK.
- Best native path: Rust `winit` + `wgpu` + `cpal`.
- Hybrid path: Tauri shell first, native renderer later.

Core features:

- Fullscreen visualizer.
- Audio input/device selection.
- Preset browser and search.
- Preset playlists and shuffle.
- Timed and beat-based automation.
- Live parameter editing.
- Fragment import/export.
- Texture pack support.
- Screenshots.
- Video/frame-sequence recording.
- Performance overlay.
- Compatibility warnings.

Advanced features:

- MIDI and OSC mapping.
- Gamepad input.
- Multi-monitor output.
- Spout/Syphon/NDI output for VJ workflows.
- Wallpaper or screensaver mode.
- Remote control API.

Exit criteria:

- A user can install and run RustyMilk as a standalone visualizer without slskR or a browser integration project.

## Phase 8: RustyMilk Studio

Goal: make authoring and debugging presets first-class.

Features:

- Preset inspector.
- Compatibility report UI.
- Expression editor with validation.
- Live parameter controls.
- Shape editor.
- Wave editor.
- Shader section editor and translator/debugger.
- Texture manager.
- Fragment library.
- Preset diff and normalize tools.
- Thumbnail generator.
- Pack builder.
- Batch compatibility dashboard.

Exit criteria:

- Preset creators can author, debug, package, and publish RustyMilk content from one app.

## Phase 9: Language SDKs And Native Interop

Goal: make RustyMilk embeddable beyond Rust and the browser.

Priority order:

- Rust crate APIs.
- TypeScript/JavaScript SDK.
- Node package for headless rendering and batch tools.
- C ABI for C, C++, C#, Godot, Unity, and plugin hosts.
- Python bindings for preset analysis, batch processing, and content tooling.
- C# wrapper for Unity and desktop apps.
- Swift/Kotlin only if mobile becomes a real target.

Exit criteria:

- Non-Rust hosts can embed the runtime or use the CLI/headless tools without depending on browser WASM assumptions.

## Phase 10: Host Integrations

Goal: reach the places visualizers are useful.

Candidate integrations:

- slskR.
- Browser and web apps.
- Electron and Tauri apps.
- OBS source plugin.
- VST/AU visualizer plugin.
- LV2 plugin for Linux audio workflows.
- TouchDesigner/Resolume/VDMX-friendly output through Spout, Syphon, or NDI.
- Wallpaper/screensaver shells.
- Game engines through C ABI or SDK wrappers.

Exit criteria:

- RustyMilk has at least one stable integration path for web, desktop, and live/video workflows.

## Audio Roadmap

RustyMilk should treat audio input as pluggable.

Work items:

- Define an audio frame contract: waveform, spectrum, bands, sample rate, channel count, timestamp.
- Support Web Audio analyzer input.
- Support custom sample streams.
- Support native audio input with `cpal`.
- Support offline WAV/MP3 analysis.
- Add configurable frequency bands.
- Add smoothing profiles.
- Add beat detection and onset detection modules.
- Add BPM estimation.
- Add deterministic replay fixtures.
- Add audio simulation/test sources.

## Compatibility Roadmap

Compatibility should be observable and testable.

Work items:

- Build a representative preset corpus.
- Add parser fixtures and round-trip tests.
- Add expression VM fixtures.
- Add shader translation fixtures.
- Add renderer snapshot tests.
- Add compatibility scoring per preset.
- Add browser compatibility matrix.
- Add native renderer compatibility matrix.
- Generate public HTML and JSON compatibility reports.
- Track unsupported features with clear diagnostics and issue links.

## Performance Roadmap

Work items:

- Define FPS and frame-time budgets per renderer.
- Add benchmark presets.
- Track parse time, runtime time, renderer time, texture upload time, and memory use.
- Add CI performance smoke thresholds.
- Add optional in-app performance overlay.
- Support low-power mode and quality presets.
- Cache parsed presets, compiled expressions, translated shaders, and uploaded textures.

## Documentation Roadmap

Docs to add:

- Getting started with Rust.
- Getting started with browser SDK.
- Getting started with standalone player.
- Preset format notes.
- Fragment format notes.
- Pack format spec.
- Plugin API spec.
- Renderer backend guide.
- Compatibility guide.
- CLI reference.
- Host integration guide.
- Architecture decision records for major API boundaries.

## Suggested Build Order

1. Finish slskR extraction and parity.
2. Stabilize `rustymilk-core` API boundaries.
3. Type and publish `packages/rustymilk-web`.
4. Define renderer-core contracts.
5. Add CLI validate/inspect/compat commands.
6. Add pack manifest schema and loader.
7. Add headless thumbnail/render path.
8. Prototype desktop player with the fastest viable shell.
9. Add plugin hooks for data packs, automation, audio, and input.
10. Build Studio features from the same SDK and CLI primitives.

## Open Design Questions

- Should the first desktop client be Tauri/Electron for speed, or native `winit`/`wgpu` for long-term architecture?
- Should renderer modularization happen before or after the first public web SDK release?
- How strict should MilkDrop compatibility be versus RustyMilk-specific extensions?
- What is the minimum stable plugin API that does not lock the project into the wrong abstraction?
- Should `.rmpack` be zip-based, folder-based, or support both?
- Which preset corpus can be used legally for compatibility and regression testing?
- What license strategy is needed for SDK/package adoption given the current AGPL license?
