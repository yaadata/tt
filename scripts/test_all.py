#!/usr/bin/env python3

import os
import subprocess
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parent.parent
LIB_ROOT = PROJECT_ROOT / "lib"
NEXTEST_COMMAND = [
    "mise",
    "exec",
    "--",
    "cargo",
    "nextest",
    "run",
    "--lib",
    "--cargo-quiet",
    "--failure-output=immediate",
    "--success-output=never",
    "--status-level=pass",
    "--no-tests=fail",
]


def run_tests(module: str | None = None) -> int:
    command = NEXTEST_COMMAND.copy()
    if module:
        command.extend(["-E", f"test(/^{module}::/)"])

    environment = os.environ.copy()
    environment["RUSTFLAGS"] = "-Awarnings"
    environment["RUST_BACKTRACE"] = "1"

    return subprocess.run(
        command,
        cwd=PROJECT_ROOT,
        env=environment,
        check=False,
    ).returncode


if __name__ == "__main__":
    raise SystemExit(run_tests())
