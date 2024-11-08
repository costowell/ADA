#!/usr/bin/env bash

# shellcheck disable=SC2103
ROOT="$(cd -- "$(dirname "$0")" > /dev/null 2>&1 || exit; cd ..; pwd -P)"

cd "$ROOT" || exit

# Setup rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

# Compile ADA
cargo build --release
