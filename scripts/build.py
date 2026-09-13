#!/usr/bin/env python3
"""Cross-compile Android ARM64 on Windows, Linux or macOS using cargo-ndk."""

import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]


def run(*args, cwd=ROOT):
    print("+ " + " ".join(map(str, args)), flush=True)
    subprocess.run(list(map(str, args)), cwd=cwd, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--binary-only",
        action="store_true",
        help="build and verify altdb ELF only; skip WebUI and module ZIP",
    )
    args = parser.parse_args()
    required = ["cargo", "cargo-ndk", "cmake", "ninja"]
    if not args.binary_only:
        required.append("npm.cmd" if os.name == "nt" else "npm")
    for tool in required:
        if not shutil.which(tool):
            raise SystemExit(f"Missing {tool} on PATH; see docs/BUILD.md prerequisites")
    version = subprocess.check_output(["cargo", "ndk", "--version"], cwd=ROOT, text=True).strip()
    if version != "cargo-ndk 4.1.2":
        raise SystemExit("Install cargo-ndk: cargo install cargo-ndk --version 4.1.2 --locked")
    run(sys.executable, ROOT / "scripts/fetch-deps.py")
    run(
        "cargo",
        "ndk",
        "-t",
        "arm64-v8a",
        "-P",
        "30",
        "build",
        "--locked",
        "--release",
        "--bin",
        "altdb",
    )
    run(sys.executable, ROOT / "scripts/package.py", "--check-binary")
    if not args.binary_only:
        npm = shutil.which(required[-1])
        run(npm, "ci", "--no-audit", "--no-fund", cwd=ROOT / "webui")
        run(npm, "run", "build", cwd=ROOT / "webui")
        run(sys.executable, ROOT / "scripts/package.py")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as error:
        raise SystemExit(error.returncode) from None
