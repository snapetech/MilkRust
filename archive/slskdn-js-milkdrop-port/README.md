# slskdN JavaScript MilkDrop Port Archive

This directory preserves the JavaScript MilkDrop/MilkDrop3 browser port from the neighboring `slskdn` repo.

It is retained as reference material for RustyMilk compatibility work:

- parser behavior
- expression VM behavior
- shader translation behavior
- WebGL2 renderer behavior
- WebGPU renderer behavior
- native visualizer wrapper behavior
- smoke, compatibility, and performance scripts
- historical design notes

This archive is not the production RustyMilk engine. Durable behavior should be migrated into RustyMilk-owned Rust crates, web SDK tests, renderer backend tests, player/studio requirements, and CLI/tooling.

Compliance status: reference-only archive. It is not included in npm package files, Cargo crates, default app bundles, or release content. Treat this directory as preserved provenance from the local `slskdn` source tree; review upstream/source ownership before redistributing it outside the RustyMilk source repository.

Source snapshot paths:

```text
src/milkdrop/
src/nativeMilkdropEngine.js
src/nativeMilkdropEngine.test.js
scripts/
docs/webgl-milkdrop3-port.md
```

Original local source:

```text
../slskdn/src/web/src/components/Player/visualizers/milkdrop/
../slskdn/src/web/src/components/Player/visualizers/nativeMilkdropEngine.js
../slskdn/src/web/scripts/
../slskdn/docs/design/webgl-milkdrop3-port.md
```
