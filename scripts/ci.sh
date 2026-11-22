#!/usr/bin/env bash
set -euo pipefail

# Run the canonical local CI checks for toybox.
#
# This script mirrors the Linux validation scope from
# `.github/workflows/state-enforcement.yml`.

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"

run_step() {
  local label="$1"
  shift
  echo "==> ${label}"
  "$@"
}

cd "${REPO_ROOT}"

run_step "Validate pinned git dependencies" ./scripts/check_pinned_git_deps.sh
run_step "Validate plugin state enforcement" ./scripts/check_clap_state_enforcement.sh
run_step "Validate declarative slot API naming" ./scripts/check_declarative_slot_api.sh
run_step "Test minimal CLAP example" cargo test -p toybox-minimal-clap
run_step "Check formatting" cargo fmt --all -- --check

lint_scope=(--workspace)
if [[ -z "${VST3_SDK_DIR:-}" ]]; then
  lint_scope+=(--exclude toybox-vst3-ffi --exclude toybox-minimal-vst3)
fi
run_step "Run clippy (gui feature)" cargo clippy "${lint_scope[@]}" --all-targets --features gui -- -D warnings
run_step "Run toybox default-feature tests" cargo test -p toybox

if [[ -n "${VST3_SDK_DIR:-}" ]]; then
  run_step "Lint toybox VST3 feature" env VST3_SDK_DIR="${VST3_SDK_DIR}" cargo clippy -p toybox --all-targets --features vst3 -- -D warnings
  run_step "Test toybox VST3 feature" env VST3_SDK_DIR="${VST3_SDK_DIR}" cargo test -p toybox --features vst3
else
  echo "==> Skipping VST3 checks (set VST3_SDK_DIR to enable)."
fi

echo "==> toybox local CI checks passed."
