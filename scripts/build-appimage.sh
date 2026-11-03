#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
[[ "$(uname -m)" == x86_64 ]] || { echo 'Build requer x86_64.' >&2; exit 1; }
TOOLS="$(realpath "${APPIMAGE_TOOLS_DIR:-$PWD/packaging/tools}")"
for tool in linuxdeploy appimagetool; do
    [[ -x "$TOOLS/$tool-x86_64.AppImage" ]] || { echo "Ferramenta ausente: $TOOLS/$tool-x86_64.AppImage. Consulte docs/APPIMAGE.md." >&2; exit 1; }
done
[[ -f "$TOOLS/runtime-x86_64" ]] || { echo "Runtime ausente; execute scripts/fetch-appimage-tools.sh" >&2; exit 1; }
cargo build --release --locked -p cctv-view
mkdir -p dist
APPDIR="$PWD/dist/CCTV-View.AppDir"
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/share/metainfo" "$APPDIR/usr/share/glib-2.0/schemas" "$APPDIR/usr/share/licenses/cctv-view"
APPIMAGE_EXTRACT_AND_RUN=1 "$TOOLS/linuxdeploy-x86_64.AppImage" --appdir "$APPDIR" --executable target/release/cctv-view --desktop-file packaging/desktop/br.dev.schuh.cctv-view.desktop --icon-file resources/icons/br.dev.schuh.cctv-view.svg
install -m755 packaging/appimage/AppRun "$APPDIR/AppRun"
install -m644 packaging/desktop/br.dev.schuh.cctv-view.metainfo.xml "$APPDIR/usr/share/metainfo/br.dev.schuh.cctv-view.appdata.xml"
install -m644 LICENSE "$APPDIR/usr/share/licenses/cctv-view/LICENSE"
install -m644 COPYRIGHT "$APPDIR/usr/share/licenses/cctv-view/COPYRIGHT"
python3 scripts/collect-rust-licenses.py "$APPDIR"
# GTK needs schemas and symbolic icons even when the destination lacks development packages.
cp -a /usr/share/glib-2.0/schemas/. "$APPDIR/usr/share/glib-2.0/schemas/"
for theme in Adwaita hicolor; do
    if [[ -d "/usr/share/icons/$theme" ]]; then mkdir -p "$APPDIR/usr/share/icons/$theme"; cp -a "/usr/share/icons/$theme/." "$APPDIR/usr/share/icons/$theme/"; fi
done
# libav may load codec modules indirectly; its linked codec libraries are collected by linuxdeploy.
# A local-only project has no public homepage. Validate every other metadata issue normally.
appstreamcli validate --no-net --override=url-homepage-missing=info packaging/desktop/br.dev.schuh.cctv-view.metainfo.xml
desktop-file-validate packaging/desktop/br.dev.schuh.cctv-view.desktop
# Update information lets AppImageUpdate/Gear Lever follow the latest GitHub Release
# through the .zsync published next to the AppImage. Empty disables it.
UPDATE_INFO="${CCTV_VIEW_UPDATE_INFO-gh-releases-zsync|rafaelhschuh|cctv-view|latest|CCTV-View-x86_64.AppImage.zsync}"
update_args=()
[[ -z "$UPDATE_INFO" ]] || update_args=(--updateinformation "$UPDATE_INFO")
# Never overwrite an executable that may still be running: publish by atomic rename.
# The staging directory keeps the final name, which the .zsync records.
staged=$(mktemp -d "$PWD/dist/.CCTV-View-XXXXXX")
trap 'rm -rf "$staged"' EXIT
# appimagetool writes the .zsync to its working directory.
(cd "$staged" && ARCH=x86_64 APPIMAGE_EXTRACT_AND_RUN=1 "$TOOLS/appimagetool-x86_64.AppImage" --no-appstream "${update_args[@]}" --runtime-file "$TOOLS/runtime-x86_64" "$APPDIR" "$staged/CCTV-View-x86_64.AppImage")
rm -f dist/CCTV-View-x86_64.AppImage.zsync
if [[ -n "$UPDATE_INFO" ]]; then
    [[ -f "$staged/CCTV-View-x86_64.AppImage.zsync" ]] || { echo 'appimagetool não gerou o .zsync.' >&2; exit 1; }
    mv -f "$staged/CCTV-View-x86_64.AppImage.zsync" dist/
fi
mv -f "$staged/CCTV-View-x86_64.AppImage" "$PWD/dist/CCTV-View-x86_64.AppImage"
(cd dist && sha256sum CCTV-View-x86_64.AppImage $([[ -f CCTV-View-x86_64.AppImage.zsync ]] && echo CCTV-View-x86_64.AppImage.zsync) > SHA256SUMS)
