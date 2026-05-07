# Third-Party RustyMilk Content Policy

RustyMilk should support old MilkDrop ecosystem content broadly, but the repo should only redistribute content with clear redistribution permission.

## Content Types

RustyMilk content imports should cover:

- `.milk` and `.milk2` presets.
- `.shape` and `.wave` fragments.
- Textures and sprites: `.png`, `.jpg`, `.jpeg`, `.webp`, `.bmp`, `.jfif`.
- Shader snippets and saved shader injections.
- MilkDrop3-style saved images embedded in presets.
- Playlist, favorites, history, rating, and search/filter metadata.
- Transition, automation, beat-change, and saved-panel settings.
- Preview thumbnails and generated compatibility reports.

## Copy Policy

- **include**: content can be committed into this repo and shipped in builds.
- **optional-download**: tooling may fetch it on user request, but it is not shipped by default.
- **link**: repo/builds may link to the upstream source, but must not copy the content.
- **review**: license/provenance is not yet clear enough to decide.
- **reject**: do not copy, fetch, package, or link as a RustyMilk source.

## License Gate

Allowed for vendoring:

- RustyMilk-owned content.
- Public domain or CC0.
- Permissive licenses such as MIT, BSD, Apache-2.0, ISC, and Zlib.
- Creative Commons licenses that permit redistribution, with attribution captured in the pack manifest.
- GPL/LGPL/AGPL-compatible content only when the content license is explicitly declared and distribution obligations are documented.

Not allowed for vendoring:

- No explicit license.
- Licenses that forbid redistribution, commercial use, modification, or sublicensing in ways that conflict with RustyMilk releases.
- Packs that are only distributed through an installer or binary without source-content license terms.
- Images, sprites, screenshots, fonts, or logos with unclear ownership.

No-license historical MilkDrop presets are common. Those should be link-only by default, even if upstream projects have used them for many years.

## Repo Model

- `content/catalog.json` is the source catalog.
- `content/third-party/<pack-id>/` is only for audited and vendorable content.
- `content/generated/` is for generated inventories, compatibility matrices, and pack indexes.
- `docs/THIRD_PARTY_CONTENT_AUDIT.generated.md` is regenerated from catalog/local scans.

## Build Model

Default builds should include RustyMilk-owned content and vetted redistributable packs only.

Optional content can be supported through:

- Runtime URL catalogs.
- User-selected local folders.
- Pack manifests that point to upstream download pages.
- Compatibility reports generated from user-provided local copies.

This keeps RustyMilk compatible with old MilkDrop packs without silently redistributing content that lacks explicit license permission.

