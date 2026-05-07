# RustyMilk Preset Packs

RustyMilk packs are folder-based content bundles for presets, textures, reusable fragments, thumbnails, data-only plugins, playlists, and automation defaults.

The current implementation supports folder packs. Zip-style `.rmpack` archives can be layered on top later without changing the manifest model.

## Layout

```text
manifest.json
presets/
textures/
fragments/
thumbnails/
plugins/
licenses/
```

Only `manifest.json` is mandatory. Asset paths in the manifest must be relative and must stay inside the pack directory.

## Manifest

```json
{
  "schemaVersion": 1,
  "id": "rustymilk-sample-pack",
  "name": "RustyMilk Sample Pack",
  "version": "0.1.0",
  "author": "RustyMilk",
  "description": "Small local fixture pack.",
  "license": "CC0-1.0",
  "requiredRustyMilkVersion": "0.1.0",
  "sourceUrls": ["https://github.com/snapetech/RustyMilk"],
  "presets": [
    {
      "id": "warm-scope",
      "title": "Warm Scope",
      "file": "presets/warm-scope.milk",
      "tags": ["scope"],
      "thumbnail": "thumbnails/warm-scope.png"
    }
  ],
  "textures": [
    { "id": "noise", "file": "textures/noise.png", "aliases": ["noise_lq"] }
  ],
  "fragments": [
    { "id": "center-polygon", "kind": "shape", "file": "fragments/center-polygon.json" }
  ],
  "plugins": [
    { "id": "default-playlist", "kind": "data", "entry": "plugins/default-playlist.json" }
  ],
  "playlist": ["warm-scope"],
  "automationDefaults": {
    "transitionSeconds": 8
  }
}
```

## Current Tooling

```bash
cargo run -p rustymilk-cli -- pack-inspect examples/sample-pack
cargo run -p rustymilk-cli -- pack-validate examples/sample-pack
```

`pack-inspect` reports manifest metadata. `pack-validate` reads referenced presets, runs compatibility analysis, checks path safety, and warns about missing optional assets such as thumbnails.

## Build Path

- `rustymilk-pack` is the shared Rust crate for pack manifests and validation.
- The CLI is the first consumer.
- The web SDK should load the same manifest shape for browser clients.
- The standalone player should accept a pack folder or future `.rmpack` archive and build a playlist from `playlist`.
- Studio should export this structure from authored presets, fragments, textures, and thumbnails.
