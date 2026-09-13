#!/usr/bin/env python3
"""Fetch the exact upstream BoringSSL archive and verify its digest."""

import hashlib
import json
import pathlib
import tarfile
import urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[1]
pin = json.loads((ROOT / "vendor/boringssl.lock.json").read_text())
destination = ROOT / "vendor/boringssl"
stamp = destination / ".altdb-revision"
if stamp.is_file() and stamp.read_text().strip() == pin["revision"]:
    print("BoringSSL revision already verified")
    raise SystemExit(0)
archive = ROOT / ".cache/boringssl.tar.gz"
archive.parent.mkdir(exist_ok=True)
if not archive.is_file() or hashlib.sha256(archive.read_bytes()).hexdigest() != pin["sha256"]:
    with urllib.request.urlopen(pin["archive"], timeout=120) as response:
        archive.write_bytes(response.read())
if hashlib.sha256(archive.read_bytes()).hexdigest() != pin["sha256"]:
    raise SystemExit("BoringSSL archive checksum mismatch")
destination.mkdir(parents=True, exist_ok=True)
with tarfile.open(archive, "r:gz") as source:
    members = source.getmembers()
    prefix = f"boringssl-{pin['revision']}/"
    for member in members:
        if member.name.rstrip("/") == prefix.rstrip("/"):
            continue
        if not member.name.startswith(prefix):
            raise SystemExit("Unexpected archive prefix")
        member.name = member.name[len(prefix) :]
        target = (destination / member.name).resolve()
        if not target.is_relative_to(destination.resolve()) or member.issym() or member.islnk():
            raise SystemExit("Unsafe archive entry")
        source.extract(member, destination, filter="data")
stamp.write_text(pin["revision"] + "\n")
print("BoringSSL source verified:", pin["revision"])
