#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

scripts/scan-bug-council-candidates.sh >"$tmp"

failed=0

while IFS='|' read -r _ class count _; do
  class="$(printf '%s' "$class" | sed 's/^ *//; s/ *$//')"
  count="$(printf '%s' "$count" | sed 's/^ *//; s/ *$//')"
  [[ -z "$class" || "$class" == "---" || "$class" == "Candidate Class" ]] && continue

  if awk -F'|' -v class="$class" -v count="$count" '
    $2 ~ class {
      c = $3
      s = $4
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", c)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", s)
      found = 1
      if (c != count || s == "New" || s == "Pending" || s == "") {
        exit 2
      }
    }
    END { if (!found) exit 3 }
  ' docs/dev/bug-council-active-backlog.md; then
    printf 'PASS active backlog tracks %s count %s\n' "$class" "$count"
  else
    printf 'FAIL active backlog missing/stale/untriaged for %s count %s\n' "$class" "$count" >&2
    failed=1
  fi
done <"$tmp"

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

printf '\nAll active backlog checks passed.\n'
