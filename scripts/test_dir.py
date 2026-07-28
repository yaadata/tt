#!/usr/bin/env python3

import argparse
import re
from pathlib import Path

from test_all import LIB_ROOT, PROJECT_ROOT, run_tests


MODULE_SEGMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


def directory_module(raw_directory: str) -> str | None:
    directory = Path(raw_directory)
    if directory.is_absolute():
        raise ValueError("directory must be repository-relative")

    resolved = (PROJECT_ROOT / directory).resolve()
    try:
        relative = resolved.relative_to(LIB_ROOT.resolve())
    except ValueError as error:
        raise ValueError("directory must be beneath lib/") from error

    if not resolved.is_dir():
        raise ValueError(f"directory does not exist: {raw_directory}")

    if any(not MODULE_SEGMENT.fullmatch(segment) for segment in relative.parts):
        raise ValueError("directory path contains a non-module segment")

    return "::".join(relative.parts) or None


def main() -> int:
    parser = argparse.ArgumentParser(description="Run tests beneath a Rust source directory.")
    parser.add_argument("directory")
    arguments = parser.parse_args()

    try:
        module = directory_module(arguments.directory)
    except ValueError as error:
        parser.error(str(error))

    return run_tests(module)


if __name__ == "__main__":
    raise SystemExit(main())
