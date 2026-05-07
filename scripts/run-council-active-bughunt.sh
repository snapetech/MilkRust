#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

out_dir="${COUNCIL_OUT_DIR:-.council}"
mkdir -p "$out_dir"
report="$out_dir/active-bughunt.md"

write_section() {
  local title="$1"
  local pattern="$2"
  shift 2

  {
    printf '\n## %s\n' "$title"
    rg -n --pcre2 --hidden --glob '!.git/**' --glob '!.council/**' --glob '!node_modules/**' "$pattern" "$@" || true
  } >>"$report"
}

cat >"$report" <<'EOF'
# RustyMilk Active Council Bughunt Candidate Report

This report is a fresh discovery queue, not proof of no bugs. A green all-phases
run means registered gates passed; every accepted row still needs a ledger row,
behavior coverage, sibling search, and a durable regression gate.
EOF

write_section "Rust filesystem/path sinks" 'std::fs::|fs::(read|write|copy|rename|remove|create)|File::(open|create)|PathBuf::from' crates tools
write_section "Rust parser/allocation/capacity sinks" 'Vec::with_capacity|String::with_capacity|HashMap::with_capacity|HashSet::with_capacity|resize\(|reserve\(|parse::<|serde_json::from|toml::from|ron::from|read_to_string' crates tools
write_section "Rust lifecycle/concurrency sinks" 'spawn\(|tokio::spawn|thread::spawn|Mutex|RwLock|Arc<|mpsc|channel\(|JoinHandle|sleep\(|timeout\(' crates tools
write_section "Rust dynamic command/process sinks" 'Command::new|Command::arg|std::process|dlopen|libloading|plugin|Plugin' crates tools
write_section "JS DOM injection sinks" 'innerHTML|outerHTML|insertAdjacentHTML|document\.write|dangerouslySetInnerHTML|eval\(|new Function\(' packages apps tools
write_section "JS storage/network/file sinks" 'localStorage|sessionStorage|fetch\(|XMLHttpRequest|FileReader|showOpenFilePicker|showSaveFilePicker|fs\.' packages apps tools
write_section "Content/catalog path and generated artifact sinks" 'content/|generated|manifest|catalog|pack|preset|\.milk|\.milk2' content docs tools crates packages apps
write_section "Release/package metadata sinks" 'version|license|repository|homepage|publish|files|package|artifact|release|workflow' package.json Cargo.toml .github docs tools

printf 'Active RustyMilk bughunt candidates saved to %s.\n' "$report"
printf 'Verdict boundary: this report is a discovery queue, not proof of no bugs.\n'
