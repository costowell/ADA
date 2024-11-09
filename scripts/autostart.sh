#!/usr/bin/env bash
#
ROOT="$(cd -- "$(dirname "$0")" > /dev/null 2>&1 || exit; cd ..; pwd -P)"
cd "$ROOT" || exit
exec cage -s -- "./target/release/ada" pn532_uart:/dev/ttyS0 /dev/ttyACM0
