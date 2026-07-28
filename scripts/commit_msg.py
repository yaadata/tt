import json
from pathlib import Path
import subprocess
import sys
from typing import Any


COMMIT_TYPES = (
    "chore",
    "docs",
    "enhance",
    "feat",
    "fix",
    "perf",
    "refactor",
    "release",
    "test",
)

NON_SCOPE_DIRECTORIES = {"fixtures"}
CONVENTIONAL_COMMIT_METADATA = "conventional-commit"


def discover_component_scopes(project_root: Path) -> set[str]:
    source_root = project_root / "lib"
    if not source_root.is_dir():
        return set()

    return {
        path.relative_to(project_root).as_posix()
        for path in source_root.iterdir()
        if path.is_dir()
        and not path.name.startswith((".", "_"))
        and path.name not in NON_SCOPE_DIRECTORIES
    }


def cargo_metadata(project_root: Path) -> dict[str, Any]:
    result = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=project_root,
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(result.stdout)


def configured_scopes(metadata: dict[str, Any]) -> set[str]:
    configuration = metadata.get(CONVENTIONAL_COMMIT_METADATA, {})
    scopes = configuration.get("scopes", [])
    return {scope for scope in scopes if isinstance(scope, str) and scope}


def discover_workspace_scopes(metadata: dict[str, Any]) -> set[str]:
    workspace_members = set(metadata.get("workspace_members", []))
    scopes = configured_scopes(metadata.get("metadata") or {})
    workspace_root = Path(metadata["workspace_root"])

    for package in metadata.get("packages", []):
        if package.get("id") not in workspace_members:
            continue

        package_metadata = package.get("metadata") or {}
        configuration = package_metadata.get(CONVENTIONAL_COMMIT_METADATA, {})
        scope = configuration.get("scope")
        if scope is None:
            package_root = Path(package["manifest_path"]).parent
            relative_package_root = package_root.relative_to(workspace_root)
            if relative_package_root == Path("."):
                continue
            scope = relative_package_root.as_posix()

        if isinstance(scope, str) and scope:
            scopes.add(scope)

    return scopes


def discover_scopes(project_root: Path) -> list[str]:
    scopes = discover_component_scopes(project_root)
    scopes.update(discover_workspace_scopes(cargo_metadata(project_root)))
    return sorted(scopes)


def main(arguments: list[str]) -> int:
    project_root = Path(__file__).resolve().parents[1]

    try:
        scopes = discover_scopes(project_root)
    except (json.JSONDecodeError, OSError, subprocess.CalledProcessError) as error:
        print(f"Unable to discover Conventional Commit scopes: {error}", file=sys.stderr)
        return 1

    command = ["conventional-pre-commit", "--strict", "--verbose"]
    if scopes:
        command.extend(["--scopes", ",".join(scopes)])
    command.extend(COMMIT_TYPES)
    command.extend(arguments)

    return subprocess.call(command, cwd=project_root)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
