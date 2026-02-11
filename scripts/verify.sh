#!/bin/sh
set -e

echo "Running Format Check..."
cargo fmt -- --check

echo "Running Linter..."
cargo clippy -- -D warnings

echo "Running Tests..."
cargo test

echo "Running Build Check..."
cargo check
