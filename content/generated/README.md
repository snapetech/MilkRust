# Generated Content Reports

This directory contains generated MilkRust content reports.

- `community-pack-summary.json`: machine-readable summary of community-unlicensed packs.
- `COMMUNITY_PACK_SUMMARY.md`: human-readable summary of the same data.
- `desktop-plugin-reports/desktop-plugin-discovery-summary.json`: machine-readable pack plugin discovery summary for desktop runners.
- `desktop-plugin-reports/*-*.json`: per-mode discovery snapshots for pack scans (`player-*`, `probe-*`, `studio-*`).
- `desktop-plugin-reports/desktop-plugin-discovery-summary.md`: optional human-readable markdown summary (`--write-markdown`).

Regenerate with:

```bash
npm run content:report-community
npm run content:report-community:compat
npm run content:report-community:compat:full
npm run desktop:plugin-report
npm run desktop:plugin-report:catalog
```
