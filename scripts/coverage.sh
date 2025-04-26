#!/usr/bin/env bash
set -euo pipefail

# Ensure LLVM tools component is installed
rustup component add llvm-tools-preview

# Install cargo-llvm-cov if missing
if ! command -v cargo-llvm-cov >/dev/null; then
  echo "Installing cargo-llvm-cov..."
  cargo install cargo-llvm-cov
fi

# Clean previous coverage artifacts
cargo llvm-cov clean --workspace

# Run coverage: include only workspace src code
cargo llvm-cov \
  --workspace \
  --ignore-filename-regex '^(?!src/|crates/[^/]+/src/).*' \
  --open
