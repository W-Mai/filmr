#!/usr/bin/env bash

echo "Check formatting"
cargo fmt -p filmr --all
cargo fmt -p filmr_app --all

echo "Build"
cargo build -p filmr --verbose --all-features --all-targets
cargo build -p filmr_app --verbose --all-features --all-targets

echo "Run tests"
cargo test -p filmr --verbose --all-features --all-targets
cargo test -p filmr_app --verbose --all-features --all-targets

echo "Clippy"
cargo clippy -p filmr --all-features --all-targets -- -D warnings
cargo clippy -p filmr_app --all-features --all-targets -- -D warnings
