# MilkRust Bug Council Phase Tracker

Started: 2026-05-07.

This tracker keeps the MilkRust council from degrading into one-off checks. The runner must execute every registered phase in one command, and a green run is a calibrated gate result, not proof that no bugs exist.

## Phases

| # | Name | Status | Owner | Exit criteria |
| --- | --- | --- | --- | --- |
| 1 | Council process docs | Done | (agent) | Severity/confidence, sibling-search, negative-space, behavior-pinning, active-backlog, and phase docs exist. |
| 2 | Fresh candidate inventory | Done | (agent) | `scripts/scan-bug-council-candidates.sh` emits current candidate counts for Rust, JS, content/catalog, lifecycle, filesystem, parser, renderer, and release surfaces. |
| 3 | Active discovery handoff | Done | (agent) | `scripts/run-council-active-bughunt.sh` writes `.council/active-bughunt.md` and states that it is a discovery queue, not a no-bug proof. |
| 4 | Active backlog drift gate | Done | (agent) | `docs/dev/bug-council-active-backlog.md` records each candidate class and `scripts/check-council-active-backlog.sh` verifies counts stay current and no row is untriaged. |
| 5 | Negative-space gate | Done | (agent) | `scripts/check-council-negative-space.sh` asserts the runner, remediation gate, package script, repo lint posture, and calibrated lens fixtures remain wired. |
| 6 | Calibrated semantic lenses | Done | (agent) | `scripts/check-milkrust-taint-lenses.sh` proves known-bad Rust and JS fixtures fire and known-good fixtures stay silent before scanning production source. |
| 7 | All-phases runner | Done | (agent) | `scripts/run-bug-council-all-phases.sh` runs inventory, active report, process gates, calibrated lenses, remediation baseline, `cargo test --workspace`, web tests, app smoke tests, pack validation, and content validation. |

## Resume Rule

1. Run `npm run check:council`.
2. If it fails, fix or ledger every finding before claiming the pass complete.
3. If it passes, report it as "all registered calibrated gates passed," not as "no bugs found."
