#!/usr/bin/env bash

ROOT="$(cd -- "$(dirname "$0")" > /dev/null 2>&1 || exit; cd ..; pwd -P)"

exec cage -s "$ROOT/target/release/ada" pn532_uart:/dev/ttyS0 /dev/ttyACM0
