# Community-Unlicensed Content

This directory is for public MilkDrop/projectM community content imported under RustyMilk's aggressive compatibility policy.

These packs are not treated as explicitly licensed third-party dependencies. They are included for compatibility testing and user-accessible preset libraries because the historical MilkDrop community shared them publicly, but many files do not carry clear per-author redistribution licenses.

Rules:

- Keep every imported pack in its own subdirectory.
- Preserve upstream README/LICENSE/provenance files where present.
- Keep `RUSTYMILK-COMMUNITY-NOTICE.md` in every imported pack.
- Exclude these packs from default builds unless a distribution explicitly opts in.
- Remove files on credible author request.

Refresh the public projectM community imports with:

```bash
npm run content:pull-community
npm run content:audit
```

