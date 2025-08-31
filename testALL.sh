#!/bin/bash
set -e

echo
echo "cleaning"
echo
# Clean build
cargo clean

echo
echo "formatting"
echo
# Run formatter
cargo fmt

echo
echo "building"
echo
# Build (should compile without errors)
cargo build

echo
echo "linting"
echo
# Run clippy with strict settings
cargo clippy --all-targets --all-features

echo
echo "running tests"
echo

# Run tests
cargo test
