#!/usr/bin/env bash
set -euo pipefail

# Scripts directory resolution
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../../../.." && pwd)"

cd "${WORKSPACE_ROOT}"

echo "==> Building and checking Rust documentation for workspace..."
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --document-private-items

echo "==> Running doctests across workspace..."
cargo test --doc --workspace

echo "==> Documentation build and doctests passed successfully!"
echo "    Output directory: ${WORKSPACE_ROOT}/target/doc"
