#!/bin/sh

# Install any necessary dependencies for the project.
# NOTE: only supports macOS and Linux.

# USAGE: `bash scripts/setup.sh`

brew install rustup
rustup-init -y
