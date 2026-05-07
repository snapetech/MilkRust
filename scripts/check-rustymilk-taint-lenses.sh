#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

tmp_bad="$(mktemp)"
tmp_good="$(mktemp)"
tmp_prod="$(mktemp)"
trap 'rm -f "$tmp_bad" "$tmp_good" "$tmp_prod"' EXIT

scan_rust() {
  perl -0ne '
    my $file = $ARGV;
    while (/let\s+(?:mut\s+)?([A-Za-z_][A-Za-z0-9_]*)[^=;]*=\s*[^;]*\.parse\(\)[^;]*;[\s\S]{0,600}?(?:Vec|String)::with_capacity\(\s*\1\s*\)/g) {
      print "$file:$.: parse-derived capacity is not obviously bounded\n";
    }
    while (/fn\s+[A-Za-z_][A-Za-z0-9_]*\([^)]*([A-Za-z_][A-Za-z0-9_]*)\s*:\s*&str[^)]*\)[\s\S]{0,400}?fs::read_to_string\(\s*\1\s*\)/g) {
      print "$file:$.: direct string path reaches filesystem without containment\n";
    }
  ' "$@" || true
}

scan_js() {
  rg -n --pcre2 \
    'innerHTML\s*=|outerHTML\s*=|insertAdjacentHTML\(' \
    "$@" || true
}

scan_rust docs/dev/council-calibration/rustymilk-taint-bad.rs >"$tmp_bad"
scan_js docs/dev/council-calibration/rustymilk-taint-bad.js >>"$tmp_bad"
if [[ ! -s "$tmp_bad" ]]; then
  printf 'RustyMilk taint lens failed: known-bad fixtures did not fire\n' >&2
  exit 1
fi

scan_rust docs/dev/council-calibration/rustymilk-taint-good.rs >"$tmp_good"
scan_js docs/dev/council-calibration/rustymilk-taint-good.js >>"$tmp_good"
if [[ -s "$tmp_good" ]]; then
  printf 'RustyMilk taint lens failed: known-good fixtures produced findings\n' >&2
  sed 's/^/  /' "$tmp_good" >&2
  exit 1
fi

scan_rust crates tools >"$tmp_prod"
scan_js packages apps tools >>"$tmp_prod"
if [[ -s "$tmp_prod" ]]; then
  printf 'RustyMilk taint lens found unadjudicated production candidates:\n' >&2
  sed 's/^/  /' "$tmp_prod" >&2
  exit 1
fi

printf 'RustyMilk calibrated taint lenses passed\n'
