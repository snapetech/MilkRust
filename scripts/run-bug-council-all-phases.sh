#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

out_dir="${COUNCIL_OUT_DIR:-.council}"
mkdir -p "$out_dir"
scan_out="$out_dir/latest-candidate-counts.md"

printf '==> Fresh candidate inventory\n'
scripts/scan-bug-council-candidates.sh | tee "$scan_out"

printf '\n==> Active discovery report\n'
scripts/run-council-active-bughunt.sh

printf '\n==> Process gates\n'
scripts/check-bug-council-all-phases.sh
scripts/check-remediation-baseline.sh

printf '\n==> Calibrated semantic lenses\n'
scripts/check-milkrust-taint-lenses.sh

printf '\n==> Product verification\n'
cargo test --workspace
npm run test:web
npm run test:apps
npm run pack:validate
npm run content:validate

printf '\nAll MilkRust bug council phases passed. Candidate counts saved to %s.\n' "$scan_out"
printf 'Council verdict boundary: this is not proof of no bugs. It means the current calibrated lenses, active backlog, process gates, Rust tests, web tests, app smoke tests, pack validation, and content validation passed.\n'
