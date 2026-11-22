"""Reproducible V4 package assembly. Only declared release files enter the .abp."""
from __future__ import annotations
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess
import sys
import zipfile

ROOT = Path(__file__).resolve().parents[1]
DIST = ROOT / "dist"


def declared_path(base: Path, value: str) -> Path:
    relative = Path(value)
    if relative.is_absolute() or ".." in relative.parts or not value:
        raise ValueError(f"Unsafe release path: {value}")
    result = (base / relative).resolve()
    if not result.is_relative_to(base.resolve()):
        raise ValueError(f"Release path leaves its directory: {value}")
    return result


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    parser.add_argument("--package", action="store_true")
    parser.add_argument("--skip-build", action="store_true")
    args = parser.parse_args()
    manifest = json.loads((ROOT / "manifest.json").read_text(encoding="utf-8"))
    for name in ("name", "icon", "version", "description", "author", "website", "entry", "wasi_version", "api_level", "permissions"):
        if name not in manifest:
            raise ValueError(f"Missing manifest field: {name}")
    if manifest["api_level"] != 4 or manifest["wasi_version"] != 3:
        raise ValueError("This project targets API Level 4 / WASI 3.")
    if not args.skip_build:
        command = ["node", str(ROOT / "scripts" / "cargo.mjs"), "build", "--locked", "--target", "wasm32-wasip2", "--lib"]
        if args.release:
            command.append("--release")
        subprocess.run(command, cwd=ROOT, check=True)
    profile = "release" if args.release else "debug"
    wasm = ROOT / "target" / "wasm32-wasip2" / profile / "deltaforce_watch_astrobox_plugin.wasm"
    if wasm.read_bytes()[:8] != b"\x00asm\x0d\x00\x01\x00":
        raise ValueError("The build output must be a WebAssembly Component.")
    DIST.mkdir(exist_ok=True)
    members = ["manifest.json", manifest["entry"], manifest["icon"], *manifest.get("additional_files", [])]
    if len(set(members)) != len(members):
        raise ValueError("Duplicate release members.")
    for member in members:
        source = wasm if member == manifest["entry"] else declared_path(ROOT, member)
        target = declared_path(DIST, member)
        if not source.is_file():
            raise ValueError(f"Declared artifact is missing: {member}")
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, target)
    report = {
        "name": manifest["name"], "version": manifest["version"],
        "apiLevel": 4, "wasiVersion": 3, "profile": profile,
        "files": [{"path": name, "bytes": (DIST / name).stat().st_size,
                   "sha256": hashlib.sha256((DIST / name).read_bytes()).hexdigest()} for name in members],
    }
    if args.package:
        archive = DIST / f"deltaforce-sync-{manifest['version']}.abp"
        # Every entry uses the same time and stable ordering to make identical builds reproducible.
        with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED, compresslevel=9) as package:
            for member in sorted(members):
                info = zipfile.ZipInfo(member.replace("\\", "/"), date_time=(2026, 1, 1, 0, 0, 0))
                info.compress_type = zipfile.ZIP_DEFLATED
                info.external_attr = 0o644 << 16
                package.writestr(info, (DIST / member).read_bytes())
        with zipfile.ZipFile(archive) as package:
            if package.testzip() is not None or set(package.namelist()) != set(members):
                raise ValueError("Package CRC or member validation failed.")
        report["package"] = {"path": archive.name, "bytes": archive.stat().st_size,
                             "sha256": hashlib.sha256(archive.read_bytes()).hexdigest()}
    report_path = ROOT / "docs" / "validation" / "build.json"
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, subprocess.CalledProcessError) as error:
        print(f"Build failed: {error}", file=sys.stderr)
        raise SystemExit(1)
