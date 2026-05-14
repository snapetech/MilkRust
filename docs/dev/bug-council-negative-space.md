# Bug Council Negative-Space Gate

The candidate scanners find call sites that exist. The negative-space gate catches required validators or process wiring that goes missing.

| Boundary | Required anchor |
| --- | --- |
| All-phases runner | `scripts/run-bug-council-all-phases.sh` |
| Package entrypoint | `check:council` in `package.json` |
| Remediation baseline | `scripts/check-remediation-baseline.sh` |
| Candidate inventory | `scripts/scan-bug-council-candidates.sh` |
| Active backlog | `docs/dev/bug-council-active-backlog.md` |
| Calibrated lenses | `scripts/check-milkrust-taint-lenses.sh` |
| Rust unsafe posture | `unsafe_code = "forbid"` in `Cargo.toml` |
| Content catalog validation | `content:validate` in `package.json` |

Every new trust boundary should add a row here and a matching assertion in `scripts/check-council-negative-space.sh`.
