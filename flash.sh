#!/bin/sh
set -eu

PROJECT_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ESPUP_ENV=${ESPUP_ENV:-"$HOME/export-esp.sh"}
ESP32_PORT=${ESP32_PORT:-/dev/cu.usbserial-0001}

if [ ! -f "$ESPUP_ENV" ]; then
    echo "Missing ESP-RS environment file: $ESPUP_ENV" >&2
    echo "Run: espup install" >&2
    exit 1
fi

# shellcheck disable=SC1090
. "$ESPUP_ENV"

cd "$PROJECT_DIR"
export RUSTUP_TOOLCHAIN=esp

cargo build --release
exec espflash flash --monitor --chip esp32 --port "$ESP32_PORT" \
    "target/xtensa-esp32-none-elf/release/esp-siren"
