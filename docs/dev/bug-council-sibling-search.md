# Bug Council Sibling Search

Every accepted finding requires a sibling sweep before closure.

For MilkRust, sibling search means checking at least:

- Rust crates under `crates/`
- browser packages under `packages/`
- app entrypoints under `apps/`
- Node tooling under `tools/`
- generated/catalog content paths under `content/` when the bug involves content ingestion
- release/package metadata when the bug involves shipped artifacts

A fix is not closed until the same bug shape is either fixed, guarded, or documented as not applicable across the sibling surface.
