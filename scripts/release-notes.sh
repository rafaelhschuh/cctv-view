#!/usr/bin/env bash
# Prints the CHANGELOG section of a version; fails when it is missing or empty.
set -euo pipefail
cd "$(dirname "$0")/.."
[[ $# -eq 1 ]] || { echo "Uso: $0 VERSÃO" >&2; exit 2; }
notes=$(awk -v v="$1" '
    /^## / { if (found) exit; found = ($2 == v) ; next }
    found { print }
' CHANGELOG.md | sed -e '/./,$!d')
[[ -n "${notes//[[:space:]]/}" ]] || { echo "CHANGELOG sem seção para $1." >&2; exit 1; }
printf '%s\n' "$notes"
