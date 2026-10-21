#!/usr/bin/env python3
"""Collect notices shipped in registry crates into the local AppDir; no network/shell use."""
import json
import shutil
import subprocess
import sys
from pathlib import Path

appdir = Path(sys.argv[1]).resolve()
output = appdir / 'usr/share/licenses/cctv-view/rust'
output.mkdir(parents=True, exist_ok=True)
metadata = json.loads(subprocess.check_output([
    'cargo', 'metadata', '--locked', '--offline', '--format-version=1',
    '--filter-platform=x86_64-unknown-linux-gnu',
], text=True))
lines = ['crate\tversion\tSPDX\tsource\tnotices']
for package in metadata['packages']:
    if not (package.get('source') or '').startswith('registry+'):
        continue
    source_dir = Path(package['manifest_path']).parent
    notices = set()
    for pattern in ('LICENSE*', 'LICENCE*', 'COPYING*', 'NOTICE*'):
        notices.update(p for p in source_dir.glob(pattern) if p.is_file())
    destination = output / f"{package['name']}-{package['version']}"
    destination.mkdir(exist_ok=True)
    for notice in sorted(notices):
        shutil.copy2(notice, destination / notice.name)
    lines.append('\t'.join([
        package['name'], package['version'], package.get('license') or 'SEE SOURCE',
        f"https://crates.io/crates/{package['name']}/{package['version']}",
        ','.join(p.name for p in sorted(notices)) or 'No top-level notice in registry archive; review source',
    ]))
(output.parent / 'RUST-DEPENDENCIES.tsv').write_text('\n'.join(lines) + '\n')
