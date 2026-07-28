#!/usr/bin/env python3

import argparse
import re
from pathlib import Path

from test_all import LIB_ROOT, PROJECT_ROOT, run_tests


MODULE_SEGMENT = re.compile(r"^[A-Za-z_][A-Za-z0-9_]*$")


def file_module(raw_file: str) -> str | None:
    file = Path(raw_file)
    if file.is_absolute():
        raise ValueError("file must be repository-relative")

    resolved = (PROJECT_ROOT / file).resolve()
    try:
        relative = resolved.relative_to(LIB_ROOT.resolve())
    except ValueError as error:
        raise ValueError("file must be beneath lib/") from error

    if not resolved.is_file():
        raise ValueError(f"file does not exist: {raw_file}")

    if relative.suffix != ".rs":
        raise ValueError("file must have a .rs extension")

    module_parts = list(relative.with_suffix("").parts)
    if module_parts == ["lib"]:
        module_parts = []
    elif module_parts and module_parts[-1] == "mod":
        module_parts.pop()

    if any(not MODULE_SEGMENT.fullmatch(segment) for segment in module_parts):
        raise ValueError("file path contains a non-module segment")

    return "::".join(module_parts) or None


def main() -> int:
    parser = argparse.ArgumentParser(description="Run tests declared in a Rust source file.")
    parser.add_argument("file")
    arguments = parser.parse_args()

    try:
        module = file_module(arguments.file)
    except ValueError as error:
        parser.error(str(error))

    return run_tests(module)


if __name__ == "__main__":
    raise SystemExit(main())
