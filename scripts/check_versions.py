#!/usr/bin/env python3
"""Check first-party transition version metadata without modifying files."""
import json
from pathlib import Path
import tomllib

ROOT = Path(__file__).resolve().parents[1]


def read_toml(path):
    return tomllib.loads((ROOT / path).read_text())


def main():
    expected = read_toml("Cargo.toml")["workspace"]["package"]["version"]
    versions = {}
    for name in ("package.json", "src-tauri/tauri.conf.json"):
        versions[name] = json.loads((ROOT / name).read_text())["version"]
    npm_lock = json.loads((ROOT / "package-lock.json").read_text())
    versions["package-lock.json"] = npm_lock["version"]
    versions["package-lock.json root"] = npm_lock["packages"][""]["version"]
    versions["src-tauri/Cargo.toml"] = read_toml("src-tauri/Cargo.toml")["package"]["version"]
    for path, names in (("Cargo.lock", ("droidops-core", "droidops-device-services", "droidops-desktop")),
                        ("src-tauri/Cargo.lock", ("droidops",))):
        packages = {package["name"]: package["version"] for package in read_toml(path)["package"]}
        for name in names:
            versions[f"{path}: {name}"] = packages[name]
    mismatches = {name: version for name, version in versions.items() if version != expected}
    if mismatches:
        for name, version in mismatches.items():
            print(f"{name}: {version}, expected {expected}")
        return 1
    print(f"Version metadata consistent: {expected} ({len(versions)} transition entries)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
