#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p packaging/tools
curl -fL --retry 2 https://github.com/linuxdeploy/linuxdeploy/releases/download/1-alpha-20251107-1/linuxdeploy-x86_64.AppImage -o packaging/tools/linuxdeploy-x86_64.AppImage
curl -fL --retry 2 https://github.com/AppImage/appimagetool/releases/download/1.9.1/appimagetool-x86_64.AppImage -o packaging/tools/appimagetool-x86_64.AppImage
curl -fL --retry 2 https://github.com/AppImage/type2-runtime/releases/download/20251108/runtime-x86_64 -o packaging/tools/runtime-x86_64
sha256sum --check packaging/appimage/tools.sha256
chmod +x packaging/tools/*.AppImage
