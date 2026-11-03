#!/bin/zsh

set -e

cargo fmt --check

cargo clippy -- -D

cargo test
