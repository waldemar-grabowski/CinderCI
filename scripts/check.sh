#!/bin/zsh

set -e

cargo fmt --check

cargo clippy -- -D warnings

cargo test
