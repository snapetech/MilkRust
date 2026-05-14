# milkrust-desktop

`milkrust-desktop` is the first native host scaffold in MilkRust.

It currently ships four binaries:

- a deterministic `MilkRustFrameSetRuntimeHost` for reuse in native loops,
- a synthetic audio profile and waveform/spectrum generators for non-audio-device scenarios,
- a reusable `DesktopPlayerEngine` API for embedding playback hosts (`next_preset`, `prev_preset`, `toggle`, etc.),
  and pluggable audio providers so hosts can supply real stream/analyzer data instead of synthetic waveform defaults.
- a headless probe binary for fast smoke and regression checks,
- a playback binary that can batch through presets and report frame statistics,
- and an optional native windowed player shell (`player-ui`) behind the `ui` cargo feature.

## Running the probe

```bash
cargo run -p milkrust-desktop --bin milkrust-desktop -- --preset examples/sample-pack/presets/warm-scope.milk --frames 120 --fps 60
```

Use `--json` for machine-readable output:

```bash
cargo run -p milkrust-desktop --bin milkrust-desktop -- --preset examples/sample-pack/presets/warm-scope.milk --frames 60 --fps 30 --json
```

The current implementation is intentionally lightweight. It is a stepping stone toward a windowed
desktop player and audio-device-backed runtime.

## Running player-ui (feature-gated)

```bash
cargo run -p milkrust-desktop --features ui --bin player-ui -- --preset examples/sample-pack/presets/warm-scope.milk --fps 60
```

```bash
cargo run -p milkrust-desktop --features ui --bin player-ui -- --pack examples/sample-pack --preset-duration 30 --no-loop --pause-start
```

To enable live audio capture, add the `audio` feature:

```bash
cargo run -p milkrust-desktop --features "ui audio" --bin player-ui -- --pack examples/sample-pack --audio-device <device-name>
```

Controls available in the window:

- `←` / `→` previous/next preset
- `Space` pause/resume
- `R` reset playback timer and restart current preset
- `ESC` or close to exit

Headless and studio binaries are currently tuned for deterministic diagnostics and compatibility
reporting; the UI shell is intentionally minimal and still rasterizes frame geometry directly for now.
