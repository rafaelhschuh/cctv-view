#!/usr/bin/env bash
# Private D-Bus and keyring; run under Xvfb. Never uses the user's credentials.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "${1:-}" != --inside ]]; then
    cctv_test_dir=$(mktemp -d)
    trap 'rm -rf "$cctv_test_dir"' EXIT
    mkdir -m700 "$cctv_test_dir/runtime" "$cctv_test_dir/data" "$cctv_test_dir/config" "$cctv_test_dir/cache" "$cctv_test_dir/keyring"
    dbus-run-session -- env XDG_RUNTIME_DIR="$cctv_test_dir/runtime" XDG_DATA_HOME="$cctv_test_dir/data" XDG_CONFIG_HOME="$cctv_test_dir/config" XDG_CACHE_HOME="$cctv_test_dir/cache" "$PWD/scripts/test-ui.sh" --inside "$cctv_test_dir"
    exit
fi
cctv_test_dir=$2
printf 'fixture-unlock-only' | gnome-keyring-daemon --foreground --unlock --components=secrets --control-directory="$cctv_test_dir/keyring" > "$cctv_test_dir/keyring.log" 2>&1 &
cctv_keyring_pid=$!
trap 'kill "$cctv_keyring_pid" 2>/dev/null || true; wait "$cctv_keyring_pid" 2>/dev/null || true' EXIT
for _ in $(seq 1 100); do
    if [[ -S "$cctv_test_dir/keyring/control" ]]; then break; fi
    sleep .05
done
[[ -S "$cctv_test_dir/keyring/control" ]] || { cat "$cctv_test_dir/keyring.log" >&2; exit 1; }
CCTV_TEST_PRIVATE_KEYRING=1 GSK_RENDERER=cairo GTK_A11Y=none GTK_USE_PORTAL=0 cargo test -p cctv-ui --locked --offline -- --ignored --test-threads=1 --nocapture
