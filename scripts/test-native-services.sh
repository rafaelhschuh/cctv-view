#!/usr/bin/env bash
# Tests credentials and audio using disposable services; never targets the user's keyring or speakers.
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "${1:-}" != --inside ]]; then
    test_dir=$(mktemp -d)
    trap 'rm -rf "$test_dir"' EXIT
    mkdir -m700 "$test_dir/runtime" "$test_dir/data" "$test_dir/config" "$test_dir/cache" "$test_dir/keyring"
    dbus-run-session -- env XDG_RUNTIME_DIR="$test_dir/runtime" XDG_DATA_HOME="$test_dir/data" XDG_CONFIG_HOME="$test_dir/config" XDG_CACHE_HOME="$test_dir/cache" "$PWD/scripts/test-native-services.sh" --inside "$test_dir"
    exit
fi
test_dir=$2
pulse_bin=${PULSEAUDIO_BIN:-pulseaudio}
command -v "$pulse_bin" >/dev/null
command -v gnome-keyring-daemon >/dev/null
printf 'fixture-unlock-only' | gnome-keyring-daemon --foreground --unlock --components=secrets --control-directory="$test_dir/keyring" > "$test_dir/keyring.log" 2>&1 &
keyring_pid=$!
cat > "$test_dir/default.pa" <<PA
.fail
load-module module-native-protocol-unix socket=$test_dir/pulse.sock auth-anonymous=1
load-module module-null-sink sink_name=cctv_test rate=48000 channels=2
set-default-sink cctv_test
PA
pulse_args=()
if [[ -n "${PULSEAUDIO_MODULE_DIR:-}" ]]; then pulse_args+=("--dl-search-path=$PULSEAUDIO_MODULE_DIR"); fi
"$pulse_bin" --daemonize=no --exit-idle-time=-1 --use-pid-file=no --disable-shm=yes -n --file="$test_dir/default.pa" --log-target="file:$test_dir/pulse.log" "${pulse_args[@]}" &
pulse_pid=$!
cleanup() { kill "$pulse_pid" "$keyring_pid" 2>/dev/null || true; wait "$pulse_pid" "$keyring_pid" 2>/dev/null || true; }
trap cleanup EXIT
for _ in $(seq 1 50); do if [[ -S "$test_dir/pulse.sock" ]]; then break; fi; sleep .1; done
[[ -S "$test_dir/pulse.sock" ]] || { cat "$test_dir/pulse.log" >&2; exit 1; }
cargo test -p cctv-storage --locked secret_service_roundtrip_and_delete -- --ignored
PULSE_SERVER="unix:$test_dir/pulse.sock" cargo test -p cctv-media --test local_stream --locked real_pcm_output_and_exclusive_mute -- --ignored
