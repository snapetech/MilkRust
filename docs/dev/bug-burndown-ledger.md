# Bug Burndown Ledger

| ID | Status | Severity | Confidence | Area | Finding | Resolution |
| --- | --- | --- | --- | --- | --- | --- |
| RM-001 | Guarded | High | Proven | Council loop | MilkRust had no repo-native all-phases bughunt runner, so future agents could confuse ad hoc checks with a full council pass. | Added all-phases runner, active inventory, backlog drift gate, negative-space gate, and calibrated Rust/JS semantic lenses. |

Status values: `New`, `Accepted`, `Fixed`, `Guarded`, `False positive`, `Out of scope`.
