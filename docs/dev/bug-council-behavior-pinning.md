# Bug Council Behavior Pinning

Text checks are allowed as presence gates, but accepted behavior fixes need executable coverage when practical.

Preferred pins:

- Rust unit/integration tests for parser, pack, renderer, and CLI behavior.
- Node tests for browser SDK and app wrappers.
- Smoke tests for desktop/player/studio command surfaces.
- Calibration fixtures for semantic lenses: one known-bad fixture that must fire and one known-good fixture that must stay silent.

When executable coverage is impossible, record why in the ledger and add the strongest available script or doc gate.
