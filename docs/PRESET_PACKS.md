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
      "sourceFormat": "milk",
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
npm run content:report-community:compat:full
```

`pack-inspect` reports manifest metadata. `pack-validate` reads referenced presets, runs compatibility analysis, checks path safety, and warns about missing optional assets such as thumbnails.

`sourceFormat` defaults from the file extension. Supported values currently include `milk`, `milk2`, and `butterchurn-json`. Native RustyMilk rendering support is for `milk` and `milk2`; `butterchurn-json` is catalog/import material until a converter or compatibility adapter is added.

The web SDK exposes both eager and lazy pack loading:

```js
import {
  loadRustyMilkPack,
  loadRustyMilkPackManifest,
  loadRustyMilkPackPresetSource,
} from '@rustymilk/web';
```

- `loadRustyMilkPack(url)` loads the manifest and every preset source. This is appropriate for small packs.
- `loadRustyMilkPackManifest(url)` loads and validates only `manifest.json`. This is the default for large community libraries.
- `loadRustyMilkPackPresetSource(preset)` fetches one preset source from a normalized manifest entry.

RustyMilk Player uses the lazy path for bundled community packs so packs with thousands of presets can populate the browser list without fetching every `.milk` file at startup.

The SDK now also supports plugin entries:

- `loadRustyMilkPackPlugins(manifest)` loads each plugin entry from `plugins/`.
- `engine.loadPlugins(plugins)` registers lifecycle hooks and data descriptors.
- pack plugin JSON descriptors can drive playlist order through a `kind: "playlist"` payload and `presetIds`.

See [`docs/PLUGIN_API.md`](PLUGIN_API.md) for the current hook surface.

## Build Path

- `rustymilk-pack` is the shared Rust crate for pack manifests and validation.
- The CLI is the first consumer.
- The web SDK loads the same manifest shape for browser clients.
- The standalone player accepts local served pack folders and builds a selectable library from generated pack indexes.
- Studio should export this structure from authored presets, fragments, textures, and thumbnails.
