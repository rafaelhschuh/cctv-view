#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mkdir -p packaging/tools
curl -fL --retry 2 https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage -o packaging/tools/linuxdeploy-x86_64.AppImage
curl -fL --retry 2 https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage -o packaging/tools/appimagetool-x86_64.AppImage
curl -fL --retry 2 https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-x86_64 -o packaging/tools/runtime-x86_64
sha256sum --check packaging/appimage/tools.sha256
chmod +x packaging/tools/*.AppImage
