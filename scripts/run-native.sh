#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd -- "$script_dir/.."

if ! command -v cargo >/dev/null 2>&1; then
  echo "Rust/Cargo is required. Install Rust from https://rustup.rs and reopen your terminal." >&2
  exit 1
fi

exec cargo run --locked -p droidops-desktop -- "$@"
