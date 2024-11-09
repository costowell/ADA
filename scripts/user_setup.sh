#!/usr/bin/env bash

# shellcheck disable=SC2103
SCRIPT_DIR="$(cd -- "$(dirname "$0")" > /dev/null 2>&1 || exit; pwd -P)"
ROOT_DIR="$(cd -- "$SCRIPT_DIR" > /dev/null 2>&1 || exit; cd .. || exit; pwd -P)"

cd "$ROOT_DIR" || exit

# Setup rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
. "$HOME/.cargo/env"

# Compile ADA
cargo build --release

# Add autostart to ~/.bashrc
echo "
if [ \"\$(tty)\" == \"/dev/tty1\" ]; then
   exec \"$SCRIPT_DIR/autostart.sh\"
fi
" >> ~/.bashrc
