#!/usr/bin/env python3
"""Collect locked dependency notices for binary/package distribution."""
import json
import pathlib
import shutil
import subprocess
import sys

destination = pathlib.Path(sys.argv[1])
destination.mkdir(parents=True, exist_ok=True)
metadata = json.loads(subprocess.check_output([
    "cargo", "metadata", "--locked", "--offline", "--format-version", "1",
    "--filter-platform", "x86_64-unknown-linux-gnu",
], text=True))
reachable = {node["id"] for node in metadata["resolve"]["nodes"]}
lines = ["# Third-party dependency notices", "", "Generated from Cargo.lock. The application license remains undecided.", ""]
for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
    if not package["source"] or package["id"] not in reachable:
        continue
    root = pathlib.Path(package["manifest_path"]).parent
    files = [p for p in root.iterdir() if p.is_file() and p.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT"))]
    if package.get("license_file"):
        files.append(root / package["license_file"])
    if not files:
        raise SystemExit(f"Missing license text for {package['name']} {package['version']}")
    folder = destination / f"{package['name']}-{package['version']}"
    folder.mkdir(exist_ok=True)
    for source in set(files):
        shutil.copy2(source, folder / source.name)
    lines.append(f"- {package['name']} {package['version']}: {package.get('license') or 'see license file'}; {package.get('repository') or package['source']}")
(destination / "THIRD_PARTY_NOTICES.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"Dependency notices collected in {destination}")
