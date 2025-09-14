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

# Run examples
echo
echo "running basic_usage example"
echo

# Run example 1
cargo run --example basic_usage

echo
echo "running send_token_example example"
echo

# Run example 2
cargo run --example send_token_example
