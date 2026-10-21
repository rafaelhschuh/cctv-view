#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
FIXTURES=$(mktemp -d)
server_pid=''
cleanup() { if [[ -n "$server_pid" ]]; then kill "$server_pid" 2>/dev/null || true; wait "$server_pid" 2>/dev/null || true; fi; rm -rf "$FIXTURES"; }
trap cleanup EXIT
cp crates/media/tests/fixtures/* "$FIXTURES/"
ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc2=size=320x180:rate=10 -t 2 -an -c:v libx264 -preset ultrafast -tune zerolatency -x264-params aud=1:repeat-headers=1 -g 10 -f h264 "$FIXTURES/video.h264"
ffmpeg -hide_banner -loglevel error -f lavfi -i testsrc2=size=320x180:rate=10 -t 2 -an -c:v libx265 -preset ultrafast -x265-params pools=1:log-level=error -f mpegts "$FIXTURES/hevc.ts"
printf '#EXTM3U\n#EXT-X-VERSION:3\n#EXT-X-TARGETDURATION:2\n#EXT-X-MEDIA-SEQUENCE:0\n#EXTINF:2.0,\nsegment.ts\n#EXT-X-ENDLIST\n' > "$FIXTURES/live.m3u8"
openssl req -x509 -newkey rsa:2048 -nodes -keyout "$FIXTURES/key.pem" -out "$FIXTURES/cert.pem" -days 1 -subj '/CN=localhost' -addext 'subjectAltName=DNS:localhost,IP:127.0.0.1' >/dev/null 2>&1
python3 tests/camera_simulator.py --fixtures "$FIXTURES" --tls-cert "$FIXTURES/cert.pem" --tls-key "$FIXTURES/key.pem" --username operator --password fixture-only > "$FIXTURES/server.log" 2>&1 &
server_pid=$!
for _ in $(seq 1 50); do if CCTV_TEST_USER=operator CCTV_TEST_PASSWORD=fixture-only target/debug/examples/probe https://127.0.0.1:18443/onvif/device_service --expect-failure
CCTV_TEST_USER=operator CCTV_TEST_PASSWORD=fixture-only target/debug/examples/probe https://127.0.0.1:18443/onvif/device_service --no-verify-tls
curl --silent --fail http://127.0.0.1:18080/events >/dev/null; then break; fi; sleep .1; done
cargo build -p cctv-media --example probe --locked
probe=target/debug/examples/probe
"$probe" http://127.0.0.1:18080/live.ts
"$probe" http://127.0.0.1:18080/snapshot.jpg
"$probe" http://127.0.0.1:18080/mjpeg
"$probe" http://127.0.0.1:18080/live.m3u8
"$probe" http://127.0.0.1:18080/hevc.ts
CCTV_TEST_USER=operator CCTV_TEST_PASSWORD=fixture-only "$probe" rtsp://127.0.0.1:18554/live
CCTV_TEST_USER=operator CCTV_TEST_PASSWORD=wrong "$probe" rtsp://127.0.0.1:18554/live --expect-failure
"$probe" rtsp://127.0.0.1:1/live --expect-failure
"$probe" https://localhost:18443/live.ts --expect-failure
# Explicit opt-out accepts this fixture certificate, without changing the secure default.
"$probe" https://localhost:18443/live.ts --no-verify-tls
"$probe" https://localhost:18443/live.m3u8 --no-verify-tls
"$probe" https://localhost:18443/snapshot.jpg --no-verify-tls
"$probe" https://localhost:18443/live.ts --expect-failure
CCTV_TEST_USER=unused CCTV_TEST_PASSWORD=unused "$probe" http://127.0.0.1:18080/live.ts --no-credentials
cargo build -p cctv-onvif --example probe --locked
CCTV_TEST_USER=operator CCTV_TEST_PASSWORD=fixture-only target/debug/examples/probe http://127.0.0.1:18080/onvif/device_service
CCTV_TEST_USER=operator CCTV_TEST_PASSWORD=fixture-only target/debug/examples/probe https://127.0.0.1:18443/onvif/device_service --expect-failure
CCTV_TEST_USER=operator CCTV_TEST_PASSWORD=fixture-only target/debug/examples/probe https://127.0.0.1:18443/onvif/device_service --no-verify-tls
curl --silent --fail http://127.0.0.1:18080/events > "$FIXTURES/events.json"
python3 - "$FIXTURES/events.json" <<'PY'
import json,sys
ops=[event['operation'] for event in json.load(open(sys.argv[1]))]
assert all(op in ops for op in ['GetDeviceInformation','GetCapabilities','GetProfiles','GetStreamUri','ContinuousMove','Stop','GetPresets','GotoPreset','SetPreset']), ops
print('SOAP operation sequence verified')
PY
