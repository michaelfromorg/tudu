#!/bin/bash

# Install any necessary dependencies for the project.
# NOTE: only supports macOS and Linux.

# USAGE: `bash scripts/setup.sh`

brew install rustup
rustup-init -y

pushd tudu || exit
cargo install cargo-insta
cargo install --path .
cargo test --all
cargo run -- --help
popd || exit
