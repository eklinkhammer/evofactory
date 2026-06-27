#!/bin/bash
set -e

echo "=== Rust unit tests ==="
PATH="$HOME/.cargo/bin:$PATH" cargo test --manifest-path rust/Cargo.toml

echo ""
echo "=== Building GDExtension ==="
PATH="$HOME/.cargo/bin:$PATH" cargo build --manifest-path rust/Cargo.toml

echo ""
echo "=== GDScript integration tests ==="
godot --headless --script res://scripts/test_runner.gd --path .
