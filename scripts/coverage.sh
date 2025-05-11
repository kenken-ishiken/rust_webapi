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

# Run coverage: include only workspace src code, exclude external dependencies
cargo llvm-cov \
  --workspace \
  --lcov --output-path lcov.info

# Filter the lcov file to include only our project code
grep -v "\.cargo" lcov.info | grep -v "rustc" > filtered_lcov.info

# Generate HTML report from filtered lcov
if ! command -v genhtml >/dev/null; then
  echo "Installing lcov for genhtml..."
  if [[ "$OSTYPE" == "darwin"* ]]; then
    brew install lcov
  else
    sudo apt-get install -y lcov
  fi
fi

genhtml filtered_lcov.info --output-directory target/llvm-cov/html
open target/llvm-cov/html/index.html
