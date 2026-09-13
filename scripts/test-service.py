#!/usr/bin/env python3
"""Linux regression for the real watchdog script and a loopback-only daemon.

Android boot properties are simulated. This does not restart a phone or verify
Android process survival; framework recovery policy has separate Rust tests.
"""

import argparse
import json
import os
from pathlib import Path
import select
import shlex
import signal
import socket
import subprocess
import tempfile
import time


def until(check, timeout=15):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        try:
            result = check()
            if result:
                return result
        except (OSError, ValueError, subprocess.SubprocessError):
            pass
        time.sleep(0.05)
    raise AssertionError("condition timed out")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", required=True, type=Path)
    parser.add_argument("--adb", required=True, type=Path)
    args = parser.parse_args()
    binary = str(args.binary.resolve())
    with tempfile.TemporaryDirectory(prefix="altdb-service-") as folder:
        root = Path(folder)
        state = root / "state"
        boot = root / "boot"
        starts = root / "starts"
        bin_dir = root / "bin"
        bin_dir.mkdir()
        boot.write_text("0\n")
        script = Path(__file__).resolve().parents[1] / "module/service.sh"
        source = script.read_text()
        assert source.count("STATE=/data/adb/altdb") == 1
        (root / "service.sh").write_text(
            source.replace("STATE=/data/adb/altdb", f"STATE={shlex.quote(str(state))}")
        )
        (bin_dir / "getprop").write_text(f"#!/bin/sh\ncat {shlex.quote(str(boot))}\n")
        (bin_dir / "altdb").write_text(
            "#!/bin/sh\n"
            # A daemon must not inherit the watchdog's lock descriptor.
            "[ ! -e /proc/self/fd/9 ] || exit 99\n"
            f"echo $$ >> {shlex.quote(str(starts))}\n"
            f"exec {shlex.quote(binary)} test-daemon {shlex.quote(str(state))}\n"
        )
        for executable in bin_dir.iterdir():
            executable.chmod(0o755)
        environment = dict(os.environ)
        environment["PATH"] = f"{bin_dir}:{environment['PATH']}"
        environment["ANDROID_USER_HOME"] = str(root / "android-user")
        Path(environment["ANDROID_USER_HOME"]).mkdir()
        with socket.socket() as probe:
            probe.bind(("127.0.0.1", 0))
            adb_port = probe.getsockname()[1]
        host = [str(args.adb.resolve()), "-P", str(adb_port)]
        watchdogs = []
        session = None

        def launch():
            process = subprocess.Popen(
                ["/bin/sh", str(root / "service.sh")], env=environment, start_new_session=True
            )
            watchdogs.append(process)
            return process

        def ctl(op):
            result = subprocess.run(
                [binary, "test-ctl", str(state), json.dumps({"op": op})],
                capture_output=True,
                text=True,
                timeout=5,
            )
            reply = json.loads(result.stdout)
            assert reply["ok"], reply
            return reply["data"]

        def running():
            status = ctl("status")
            return status if status["state"] == "running" else None

        def adb(*command):
            return subprocess.run(
                host + list(command),
                env=environment,
                check=True,
                capture_output=True,
                timeout=20,
            ).stdout

        def exchange(data):
            assert session.poll() is None
            session.stdin.write(data)
            assert select.select([session.stdout], [], [], 5)[0], "shell response timed out"
            assert os.read(session.stdout.fileno(), len(data)) == data

        try:
            watchdog = launch()
            until(lambda: (state / "service.lock").exists())
            until(
                lambda: subprocess.run(
                    ["flock", "-n", str(state / "service.lock"), "true"], timeout=2
                ).returncode
                == 1
            )
            duplicates = [launch() for _ in range(4)]
            assert all(process.wait(timeout=3) == 0 for process in duplicates)
            assert not starts.exists() and watchdog.poll() is None
            print("PASS watchdog single instance before boot completed", flush=True)

            boot.write_text("1\n")
            status = until(running)
            address = status["endpoints"][0]
            pid = int(starts.read_text().strip())
            pairing = ctl("pair_start")["pairing"]
            assert b"Successfully paired" in adb("pair", pairing["endpoints"][0], pairing["code"])
            assert b"connected" in adb("connect", address)
            session = subprocess.Popen(
                host + ["-s", address, "shell", "-T", "cat"],
                env=environment,
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.DEVNULL,
                bufsize=0,
            )
            exchange(b"a")
            boot.write_text("0\n")
            watchdog.send_signal(signal.SIGHUP)
            duplicates = [launch() for _ in range(4)]
            assert all(process.wait(timeout=3) == 0 for process in duplicates)
            time.sleep(2)
            exchange(b"b")
            boot.write_text("1\n")
            exchange(b"c")
            assert starts.read_text().splitlines() == [str(pid)]
            assert ctl("status")["endpoints"] == [address]
            print(
                "PASS boot property reset / duplicate scripts / HUP preserve TLS shell and port",
                flush=True,
            )

            session.stdin.close()
            session.wait(timeout=5)
            session.stdout.close()
            session = None
            os.kill(pid, signal.SIGKILL)
            until(lambda: len(starts.read_text().splitlines()) == 2)
            until(running)
            assert watchdog.poll() is None
            print("PASS crash restart and watchdog lock not inherited by daemon", flush=True)

            (root / "disable").touch()
            assert watchdog.wait(timeout=10) == 0
            assert not (state / "control.sock").exists()
            (root / "disable").unlink()
            watchdog = launch()
            until(running)
            (root / "remove").touch()
            assert watchdog.wait(timeout=10) == 0
            assert not (state / "control.sock").exists()
            print("PASS disable / re-enable / remove and automatic lock release", flush=True)
        finally:
            (root / "disable").touch()
            if session:
                session.kill()
                session.wait()
            for process in watchdogs:
                if process.poll() is None:
                    process.terminate()
                    process.wait(timeout=10)
            subprocess.run(host + ["kill-server"], env=environment, capture_output=True, timeout=10)


if __name__ == "__main__":
    main()
