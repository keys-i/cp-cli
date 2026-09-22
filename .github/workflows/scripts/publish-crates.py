#!/usr/bin/env python3

from __future__ import annotations

import argparse
import email.utils
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile
import time
from urllib.error import HTTPError
from urllib.request import Request, urlopen


ROOT = Path(__file__).resolve().parents[3]
PLATFORM_PACKAGES = (
    ("cp-cli-platform-codechef", "platforms/codechef/Cargo.toml"),
    ("cp-cli-platform-codeforces", "platforms/codeforces/Cargo.toml"),
    ("cp-cli-platform-hackerearth", "platforms/hackerearth/Cargo.toml"),
    ("cp-cli-platform-hackerrank", "platforms/hackerrank/Cargo.toml"),
    ("cp-cli-platform-leetcode", "platforms/leetcode/Cargo.toml"),
    ("cp-cli-platform-project-euler", "platforms/project-euler/Cargo.toml"),
)
INCOMPLETE_3_1_8_PACKAGES = (
    "cp-cli-platform-codechef",
    "cp-cli-platform-codeforces",
    "cp-cli-platform-hackerearth",
    "cp-cli-platform-hackerrank",
    "cp-cli-platform-leetcode",
)
VERSION_ROW = re.compile(r"^\| `([0-9]+\.[0-9]+\.[0-9]+)` \|")
RETRY_AT = re.compile(r"try again after (.+? GMT)(?:\s|$)", re.IGNORECASE)


class ReleaseError(RuntimeError):
    pass


def package_version(manifest: Path) -> str:
    in_package = False
    for line in manifest.read_text(encoding="utf-8").splitlines():
        if line == "[package]":
            in_package = True
            continue
        if line.startswith("["):
            in_package = False
        if in_package and line.startswith("version = "):
            return line.split('"', 2)[1]
    raise ReleaseError(f"could not read a package version from {manifest}")


def changelog_versions(changelog: Path, current: str) -> list[str]:
    versions = [
        match.group(1)
        for line in changelog.read_text(encoding="utf-8").splitlines()
        if (match := VERSION_ROW.match(line))
    ]
    if not versions or versions[0] != "0.1.0" or versions[-1] != current:
        raise ReleaseError(
            f"{changelog} must list 0.1.0 through {current} in publish order"
        )
    if len(versions) != len(set(versions)):
        raise ReleaseError(f"{changelog} contains duplicate versions")
    if versions != sorted(versions, key=lambda value: tuple(map(int, value.split(".")))):
        raise ReleaseError(f"{changelog} versions are not in ascending order")
    return versions


def replace_manifest_version(text: str, version: str) -> str:
    lines = text.splitlines(keepends=True)
    in_package = False
    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped == "[package]":
            in_package = True
            continue
        if stripped.startswith("["):
            in_package = False
        if in_package and stripped.startswith("version = "):
            suffix = "\n" if line.endswith("\n") else ""
            lines[index] = f'version = "{version}"{suffix}'
            return "".join(lines)
    raise ReleaseError("root manifest has no package version")


def replace_lock_version(text: str, version: str) -> str:
    lines = text.splitlines(keepends=True)
    in_root_package = False
    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped == "[[package]]":
            in_root_package = False
            continue
        if stripped == 'name = "cp-cli"':
            in_root_package = True
            continue
        if in_root_package and stripped.startswith("version = "):
            suffix = "\n" if line.endswith("\n") else ""
            lines[index] = f'version = "{version}"{suffix}'
            return "".join(lines)
    raise ReleaseError("lockfile has no cp-cli package version")


def prepare_workspace(destination: Path) -> None:
    for name in ("Cargo.toml", "Cargo.lock", "README.md", "LICENSE"):
        shutil.copy2(ROOT / name, destination / name)
    shutil.copytree(ROOT / "src", destination / "src")
    shutil.copytree(
        ROOT / "platforms",
        destination / "platforms",
        ignore=shutil.ignore_patterns("target", "__pycache__"),
    )


def sparse_index_path(crate: str) -> str:
    length = len(crate)
    if length == 1:
        return f"1/{crate}"
    if length == 2:
        return f"2/{crate}"
    if length == 3:
        return f"3/{crate[0]}/{crate}"
    return f"{crate[:2]}/{crate[2:4]}/{crate}"


def retry_delay(
    message: str,
    now: float | None = None,
    retry_after: str | None = None,
) -> int:
    current = time.time() if now is None else now
    if retry_after:
        if retry_after.isdigit():
            return max(5, int(retry_after) + 1)
        try:
            retry_at = email.utils.parsedate_to_datetime(retry_after).timestamp()
            return max(5, int(retry_at - current) + 5)
        except (TypeError, ValueError, OverflowError):
            pass
    match = RETRY_AT.search(message)
    if not match:
        return int(os.environ.get("CP_CLI_PUBLISH_RETRY_SECONDS", "60"))
    retry_at = email.utils.parsedate_to_datetime(match.group(1)).timestamp()
    return max(5, int(retry_at - current) + 5)


class Registry:
    def __init__(self, current: str) -> None:
        self.user_agent = (
            f"cp-cli-release/{current} (https://github.com/keys-i/cp-cli)"
        )
        self.known: dict[str, set[str]] = {}

    def request(self, url: str, missing_ok: bool) -> bytes | None:
        max_attempts = int(os.environ.get("CP_CLI_PUBLISH_MAX_ATTEMPTS", "8"))
        max_wait = int(os.environ.get("CP_CLI_PUBLISH_MAX_WAIT_SECONDS", "3600"))
        for attempt in range(1, max_attempts + 1):
            request = Request(url, headers={"User-Agent": self.user_agent})
            try:
                with urlopen(request, timeout=30) as response:
                    return response.read()
            except HTTPError as error:
                if error.code == 404 and missing_ok:
                    return None
                if error.code != 429:
                    raise ReleaseError(
                        f"crates.io returned HTTP {error.code} while checking {url}"
                    ) from error
                detail = error.read().decode("utf-8", errors="replace")
                retry_after = error.headers.get("Retry-After")
                delay = retry_delay(detail, retry_after=retry_after)
                if delay > max_wait:
                    raise ReleaseError(
                        f"crates.io asked to wait {delay}s, above the {max_wait}s safety limit"
                    ) from error
                print(
                    f"crates.io status check rate-limited ({attempt}/{max_attempts}); "
                    f"retrying in {delay}s"
                )
                time.sleep(delay)
        raise ReleaseError(f"crates.io kept rate-limiting the status check for {url}")

    def versions(self, crate: str, refresh: bool = False) -> set[str]:
        if crate in self.known and not refresh:
            return self.known[crate]
        content = self.request(
            f"https://index.crates.io/{sparse_index_path(crate)}", missing_ok=True
        )
        versions = (
            {
                json.loads(line)["vers"]
                for line in content.decode("utf-8").splitlines()
                if line
            }
            if content is not None
            else set()
        )
        self.known[crate] = versions
        return versions

    def exact_version_exists(self, crate: str, version: str) -> bool:
        return (
            self.request(
                f"https://crates.io/api/v1/crates/{crate}/{version}",
                missing_ok=True,
            )
            is not None
        )

    def wait_until_indexed(self, crate: str, version: str) -> None:
        timeout = int(os.environ.get("CP_CLI_INDEX_WAIT_SECONDS", "300"))
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            if version in self.versions(crate, refresh=True):
                return
            print(f"Waiting for crates.io to index {crate} {version}")
            time.sleep(5)
        raise ReleaseError(f"crates.io did not index {crate} {version} within {timeout}s")


def run_cargo(command: list[str], cwd: Path) -> tuple[int, str]:
    process = subprocess.Popen(
        command,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
    )
    output: list[str] = []
    if process.stdout is None:
        raise ReleaseError("cargo output was unavailable")
    for line in process.stdout:
        print(line, end="", flush=True)
        output.append(line)
    return process.wait(), "".join(output)


def publish_package(
    registry: Registry,
    workspace: Path,
    package: str,
    version: str,
    dry_run: bool,
) -> None:
    if not dry_run and version in registry.versions(package):
        print(f"{package} {version} is already published")
        return

    command = [
        "cargo",
        "publish",
        "--locked",
        "--package",
        package,
        "--manifest-path",
        str(workspace / "Cargo.toml"),
    ]
    if dry_run:
        command.append("--dry-run")

    max_attempts = int(os.environ.get("CP_CLI_PUBLISH_MAX_ATTEMPTS", "8"))
    max_wait = int(os.environ.get("CP_CLI_PUBLISH_MAX_WAIT_SECONDS", "3600"))
    for attempt in range(1, max_attempts + 1):
        print(f"Publishing {package} {version} ({attempt}/{max_attempts})")
        return_code, output = run_cargo(command, workspace)
        if return_code == 0:
            if not dry_run:
                registry.versions(package).add(version)
            return
        if not dry_run and registry.exact_version_exists(package, version):
            registry.versions(package).add(version)
            print(f"{package} {version} reached crates.io despite Cargo's local error")
            return
        if "429 Too Many Requests" not in output:
            raise ReleaseError(f"cargo publish failed for {package} {version}")
        delay = retry_delay(output)
        if delay > max_wait:
            raise ReleaseError(
                f"crates.io asked to wait {delay}s, above the {max_wait}s safety limit"
            )
        print(f"crates.io rate limit reached; retrying in {delay}s")
        time.sleep(delay)
    raise ReleaseError(f"cargo publish kept rate-limiting {package} {version}")


def yank_package(workspace: Path, package: str, version: str, dry_run: bool) -> None:
    if dry_run:
        print(f"Would yank {package} {version}")
        return
    command = ["cargo", "yank", package, "--version", version]
    max_attempts = int(os.environ.get("CP_CLI_PUBLISH_MAX_ATTEMPTS", "8"))
    max_wait = int(os.environ.get("CP_CLI_PUBLISH_MAX_WAIT_SECONDS", "3600"))
    for attempt in range(1, max_attempts + 1):
        print(f"Yanking {package} {version} ({attempt}/{max_attempts})")
        return_code, output = run_cargo(command, workspace)
        if return_code == 0 or "already yanked" in output.lower():
            return
        if "429 Too Many Requests" not in output:
            raise ReleaseError(f"cargo yank failed for {package} {version}")
        delay = retry_delay(output)
        if delay > max_wait:
            raise ReleaseError(
                f"crates.io asked to wait {delay}s, above the {max_wait}s safety limit"
            )
        print(f"crates.io rate limit reached; retrying in {delay}s")
        time.sleep(delay)
    raise ReleaseError(f"cargo yank kept rate-limiting {package} {version}")


def set_root_version(
    workspace: Path,
    manifest: str,
    lockfile: str,
    readme: str,
    current: str,
    version: str,
) -> None:
    (workspace / "Cargo.toml").write_text(
        replace_manifest_version(manifest, version), encoding="utf-8"
    )
    (workspace / "Cargo.lock").write_text(
        replace_lock_version(lockfile, version), encoding="utf-8"
    )
    (workspace / "README.md").write_text(
        readme.replace(current, version), encoding="utf-8"
    )


def self_test() -> None:
    manifest = '[package]\nname = "cp-cli"\nversion = "3.1.8"\n\n[workspace]\n'
    lockfile = '[[package]]\nname = "cp-cli"\nversion = "3.1.8"\n'
    assert 'version = "0.1.0"' in replace_manifest_version(manifest, "0.1.0")
    assert 'version = "0.1.0"' in replace_lock_version(lockfile, "0.1.0")
    assert sparse_index_path("cp-cli") == "cp/-c/cp-cli"
    timestamp = email.utils.parsedate_to_datetime(
        "Tue, 22 Sep 2026 22:10:29 GMT"
    ).timestamp()
    assert retry_delay(
        "Please try again after Tue, 22 Sep 2026 22:10:29 GMT and see docs",
        now=timestamp - 20,
    ) == 25
    assert retry_delay("", retry_after="12") == 13
    current = package_version(ROOT / "Cargo.toml")
    versions = changelog_versions(ROOT / "docs/CHANGELOG.md", current)
    assert versions[0] == "0.1.0"
    assert versions[-1] == current
    with tempfile.TemporaryDirectory(prefix="cp-cli-publish-test-") as temporary:
        workspace = Path(temporary)
        prepare_workspace(workspace)
        manifest_text = (workspace / "Cargo.toml").read_text(encoding="utf-8")
        lock_text = (workspace / "Cargo.lock").read_text(encoding="utf-8")
        readme_text = (workspace / "README.md").read_text(encoding="utf-8")
        for version in versions:
            set_root_version(
                workspace,
                manifest_text,
                lock_text,
                readme_text,
                current,
                version,
            )
            assert package_version(workspace / "Cargo.toml") == version
            assert f'name = "cp-cli"\nversion = "{version}"' in (
                workspace / "Cargo.lock"
            ).read_text(encoding="utf-8")
        for _package, manifest_path in PLATFORM_PACKAGES:
            platform = workspace / Path(manifest_path).parent
            assert (platform / "README.md").is_file()
            assert package_version(platform / "Cargo.toml") == current
            assert 'readme = "README.md"' in (platform / "Cargo.toml").read_text(
                encoding="utf-8"
            )
    print("publish-crates self-test passed")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Publish cp-cli crates in dependency order with resumable rate-limit handling"
    )
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--version", help="publish the checked-in release version")
    mode.add_argument(
        "--history",
        action="store_true",
        help="publish cp-cli versions from the changelog after current platform crates",
    )
    parser.add_argument(
        "--dry-run", action="store_true", help="run Cargo's checks without uploading"
    )
    parser.add_argument(
        "--yank-incomplete-3-1-8",
        action="store_true",
        help="yank the five platform archives published before READMEs were added",
    )
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        return 0
    if args.yank_incomplete_3_1_8 and not args.history:
        raise ReleaseError("--yank-incomplete-3-1-8 requires --history")

    current = package_version(ROOT / "Cargo.toml")
    if args.version and args.version != current:
        raise ReleaseError(
            f"requested version {args.version} does not match workspace version {current}"
        )
    versions = (
        changelog_versions(ROOT / "docs/CHANGELOG.md", current)
        if args.history
        else [current]
    )
    registry = Registry(current)

    with tempfile.TemporaryDirectory(prefix="cp-cli-publish-") as temporary:
        workspace = Path(temporary)
        prepare_workspace(workspace)
        manifest = (workspace / "Cargo.toml").read_text(encoding="utf-8")
        lockfile = (workspace / "Cargo.lock").read_text(encoding="utf-8")
        readme = (workspace / "README.md").read_text(encoding="utf-8")

        for package, _manifest in PLATFORM_PACKAGES:
            publish_package(registry, workspace, package, current, args.dry_run)
        if not args.dry_run:
            for package, _manifest in PLATFORM_PACKAGES:
                registry.wait_until_indexed(package, current)
        for version in versions:
            set_root_version(
                workspace, manifest, lockfile, readme, current, version
            )
            publish_package(registry, workspace, "cp-cli", version, args.dry_run)
        if args.yank_incomplete_3_1_8:
            for package in INCOMPLETE_3_1_8_PACKAGES:
                if args.dry_run or "3.1.8" in registry.versions(
                    package, refresh=True
                ):
                    yank_package(workspace, package, "3.1.8", args.dry_run)
                else:
                    print(f"{package} 3.1.8 was never published; nothing to yank")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ReleaseError, subprocess.SubprocessError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
