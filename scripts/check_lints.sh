#!/usr/bin/env bash
set -euo pipefail

# Run full clippy checks, including the VST3 feature path where stream and ABI
# code is compiled.

if [[ -z "${VST3_SDK_DIR:-}" ]]; then
  echo "Set VST3_SDK_DIR to the VST3 SDK root for all-feature linting." >&2
  exit 1
fi

repo_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repo_root}"

bash scripts/check_declarative_slot_api.sh

cargo clippy --workspace --all-targets --all-features -- -D warnings

cargo clippy --features vst3 --all-targets -- -D warnings
