# Gaps & Roadmap Execution Plan

This file captures the gap analysis between what exists and what the
[`ROADMAP.md`](docs/ROADMAP.md) defines, ordered by priority.
Each gap references the concrete file(s) and crate(s) that need work.

---

## Phase 1: Core Engine Foundation (milkrust-core split)

**Status:** `milkrust-core` is a single 6,920-line lib with all parsing,
VM, runtime, compatibility, geometry, and WebGPU batching.

**Gaps:**
- No `milkrust-preset` crate (preset docs, parsing, fragments, serialization).
- No `milkrust-expr` crate (expression parser, VM, scope, math compatibility).
- No `milkrust-runtime` crate (frame runtime, transitions, automation, deterministic replay).
- Structured diagnostics (error codes, source locations, remediation hints) missing.
- Preset normalization/migration helpers missing.

**Plan:** Extract in this order: expr → preset → runtime, keeping `milkrust-core`
as a compatibility re-export crate during the transition.

---

## Phase 2: Renderer Backend Architecture

**Status:** `milkrust-renderer-core` defines contracts;
`milkrust-renderer-headless` provides stats backend.

**Gaps:**
- `milkrust-renderer-webgl` — WebGL2 renderer lives inside
  `crates/milkrust-wasm/src/renderer.rs` (1,538 lines). Needs extraction.
- `milkrust-renderer-canvas` — 2D fallback/debug renderer missing entirely.
- `milkrust-renderer-wgpu` — Native/WebGPU renderer missing entirely.
- Golden-frame/perceptual snapshot tests missing.
- Renderer capability reporting defined but not fully fleshed out.

---

## Phase 3: Web SDK (TypeScript conversion)

**Status:** `packages/milkrust-web` exists as JS with typed `.d.ts` files.
React bindings scaffolded. Web component working.

**Gaps:**
- Not yet converted to TypeScript (ROADMAP Phase 3).
- No `browser-basic` or `react-basic` examples.
- SDK lifecycle documentation missing.

---

## Phase 9: Language SDKs and Native Interop

**Status:** Rust crate APIs exist. TypeScript SDK (JS) exists.

**Gaps:**
- `milkrust-node` package missing (Node.js headless/batch).
- C ABI layer missing (C, C++, C#, Godot, Unity integration).
- Python bindings, C# wrapper, Swift/Kotlin all future.

---

## Desktop UI / Player Enhancements

**Status:** `crates/milkrust-desktop` has headless probe, player-api, and
a `player-ui` shell behind feature gates. Browser player has playlist lifecycle.

**Gaps:**
- `player-ui` is a prototype: no preset browser, no search, no parameter editor,
  no compatibility warnings UI, no audio device selection UI.
- Preset libraries and favorites workflows missing from both player and studio.

---

## Quick Wins

- **Missing thumbnails:** `pack-validate` warns `thumbnails/warm-scope.png is missing`.
  A thumbnail generator CLI command would fix this.
- **TypeScript tooling:** No `tsconfig.json`, ESLint, or Prettier.
- **GitHub templates:** No issue/PR templates, no SECURITY.md.
