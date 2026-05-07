#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

count_rg() {
  local pattern="$1"
  shift

  rg -n --pcre2 --hidden --glob '!.git/**' --glob '!.council/**' --glob '!node_modules/**' "$pattern" "$@" 2>/dev/null | wc -l | tr -d ' '
}

printf '# RustyMilk Council Scan Candidate Counts\n\n'
printf 'Generated from local source patterns. Counts are candidate lines, not confirmed bugs.\n\n'
printf '| Candidate Class | Count |\n'
printf '| --- | ---: |\n'
printf '| Rust filesystem/path sinks | %s |\n' "$(count_rg 'std::fs::|fs::(read|write|copy|rename|remove|create)|File::(open|create)|PathBuf::from|Command::new' crates tools)"
printf '| Rust parser/allocation/capacity sinks | %s |\n' "$(count_rg 'Vec::with_capacity|String::with_capacity|HashMap::with_capacity|HashSet::with_capacity|resize\\(|reserve\\(|parse::<|serde_json::from|toml::from|ron::from|read_to_string' crates tools)"
printf '| Rust lifecycle/concurrency sinks | %s |\n' "$(count_rg 'spawn\\(|tokio::spawn|thread::spawn|Mutex|RwLock|Arc<|mpsc|channel\\(|JoinHandle|sleep\\(|timeout\\(' crates tools)"
printf '| Rust dynamic command/process sinks | %s |\n' "$(count_rg 'Command::new|Command::arg|std::process|dlopen|libloading|plugin|Plugin' crates tools)"
printf '| JS DOM injection sinks | %s |\n' "$(count_rg 'innerHTML|outerHTML|insertAdjacentHTML|document\\.write|dangerouslySetInnerHTML|eval\\(|new Function\\(' packages apps tools)"
printf '| JS storage/network/file sinks | %s |\n' "$(count_rg 'localStorage|sessionStorage|fetch\\(|XMLHttpRequest|FileReader|showOpenFilePicker|showSaveFilePicker|fs\\.' packages apps tools)"
printf '| Content/catalog path and generated artifact sinks | %s |\n' "$(count_rg 'content/|generated|manifest|catalog|pack|preset|\\.milk|\\.milk2' content docs tools crates packages apps)"
printf '| Release/package metadata sinks | %s |\n' "$(count_rg 'version|license|repository|homepage|publish|files|package|artifact|release|workflow' package.json Cargo.toml .github docs tools)"
