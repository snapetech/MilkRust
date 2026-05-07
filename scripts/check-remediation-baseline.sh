#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

scripts/check-council-negative-space.sh
scripts/check-council-active-backlog.sh

printf '\nRustyMilk remediation baseline checks passed.\n'
