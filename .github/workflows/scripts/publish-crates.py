#!/usr/bin/env python3

from __future__ import annotations

import argparse
from datetime import datetime, timezone
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
NUMERIC_VERSION = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")
RETRY_AT = re.compile(r"try again after (.+? GMT)(?:\s|$)", re.IGNORECASE)
RATE_LIMITED = re.compile(
    r"(?:\b(?:status|http)\s+429\b|too many requests)", re.IGNORECASE
)
VERSION_QUOTA = re.compile(
    r"too many versions of this crate in the last 24 hours", re.IGNORECASE
)
VERSION_QUOTA_LIMIT = 20
VERSION_QUOTA_WINDOW_SECONDS = 86_400
DEFAULT_MAX_WAIT_SECONDS = VERSION_QUOTA_WINDOW_SECONDS + 10


class ReleaseError(RuntimeError):
    pass


class PublicationDeferred(ReleaseError):
    def __init__(self, package: str, version: str, delay: int) -> None:
        self.package = package
        self.version = version
        self.delay = delay
        super().__init__(f"{package} {version} is deferred for {delay}s")


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


def version_key(value: str) -> tuple[int, ...]:
    return tuple(map(int, value.split(".")))


def older_versions(versions: set[str], current: str) -> list[str]:
    current_key = version_key(current)
    return sorted(
        (
            version
            for version in versions
            if NUMERIC_VERSION.fullmatch(version) and version_key(version) < current_key
        ),
        key=version_key,
    )


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
    fallback = int(os.environ.get("CP_CLI_PUBLISH_RETRY_SECONDS", "60"))
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
        return fallback
    try:
        retry_at = email.utils.parsedate_to_datetime(match.group(1)).timestamp()
    except (TypeError, ValueError, OverflowError):
        return fallback
    return max(5, int(retry_at - current) + 5)


def is_rate_limited(message: str) -> bool:
    return RATE_LIMITED.search(message) is not None


def version_quota_delay(created_at: list[str], now: float | None = None) -> int:
    current = time.time() if now is None else now
    timestamps: list[float] = []
    for value in created_at:
        try:
            published_at = datetime.fromisoformat(value.replace("Z", "+00:00"))
            if published_at.tzinfo is None:
                raise ValueError("publication timestamp has no timezone")
            timestamps.append(published_at.timestamp())
        except (AttributeError, TypeError, ValueError) as error:
            raise ReleaseError("crates.io returned an invalid publication timestamp") from error
    recent = sorted(
        timestamp
        for timestamp in timestamps
        if timestamp > current - VERSION_QUOTA_WINDOW_SECONDS
    )
    if len(recent) < VERSION_QUOTA_LIMIT:
        raise ReleaseError(
            "crates.io reported a 24-hour version quota without 20 recent releases"
        )
    reopens_at = (
        recent[len(recent) - VERSION_QUOTA_LIMIT] + VERSION_QUOTA_WINDOW_SECONDS
    )
    return max(5, int(reopens_at - current) + 6)


class Registry:
    def __init__(self, current: str) -> None:
        self.user_agent = (
            f"cp-cli-release/{current} (https://github.com/keys-i/cp-cli)"
        )
        self.known: dict[str, set[str]] = {}

    def request(self, url: str, missing_ok: bool) -> bytes | None:
        max_attempts = int(os.environ.get("CP_CLI_PUBLISH_MAX_ATTEMPTS", "8"))
        max_wait = int(
            os.environ.get(
                "CP_CLI_PUBLISH_MAX_WAIT_SECONDS", DEFAULT_MAX_WAIT_SECONDS
            )
        )
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

    def version_quota_delay(self, crate: str) -> int:
        content = self.request(
            f"https://crates.io/api/v1/crates/{crate}/versions?per_page=100",
            missing_ok=False,
        )
        if content is None:
            raise ReleaseError(f"crates.io returned no versions for {crate}")
        try:
            payload = json.loads(content)
            versions = payload["versions"]
            created_at = [version["created_at"] for version in versions]
        except (KeyError, TypeError, json.JSONDecodeError) as error:
            raise ReleaseError(
                f"crates.io returned invalid version metadata for {crate}"
            ) from error
        return version_quota_delay(created_at)

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
        errors="replace",
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
    max_wait = int(
        os.environ.get("CP_CLI_PUBLISH_MAX_WAIT_SECONDS", DEFAULT_MAX_WAIT_SECONDS)
    )
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
        if VERSION_QUOTA.search(output):
            delay = registry.version_quota_delay(package)
            if delay > max_wait:
                raise ReleaseError(
                    f"crates.io asked to wait {delay}s, above the {max_wait}s safety limit"
                )
            raise PublicationDeferred(package, version, delay)
        elif is_rate_limited(output):
            delay = retry_delay(output)
            print(f"crates.io rate limit reached; retrying in {delay}s", flush=True)
        else:
            raise ReleaseError(f"cargo publish failed for {package} {version}")
        if delay > max_wait:
            raise ReleaseError(
                f"crates.io asked to wait {delay}s, above the {max_wait}s safety limit"
            )
        time.sleep(delay)
    raise ReleaseError(f"cargo publish kept rate-limiting {package} {version}")


def publish_queue(
    registry: Registry,
    workspace: Path,
    releases: list[tuple[str, str]],
    dry_run: bool,
    *,
    publish=publish_package,
    wait=time.sleep,
) -> None:
    pending = releases
    while pending:
        deferred: list[PublicationDeferred] = []
        for package, version in pending:
            try:
                publish(registry, workspace, package, version, dry_run)
            except PublicationDeferred as error:
                retry_at = time.strftime(
                    "%Y-%m-%d %H:%M:%S UTC",
                    time.gmtime(time.time() + error.delay),
                )
                print(
                    f"{error.package} {error.version} deferred until {retry_at}",
                    flush=True,
                )
                deferred.append(error)
        if not deferred:
            return
        delay = min(error.delay for error in deferred)
        wait(delay)
        pending = [(error.package, error.version) for error in deferred]


def yank_package(workspace: Path, package: str, version: str, dry_run: bool) -> None:
    if dry_run:
        print(f"Would yank {package} {version}")
        return
    command = ["cargo", "yank", package, "--version", version]
    max_attempts = int(os.environ.get("CP_CLI_PUBLISH_MAX_ATTEMPTS", "8"))
    max_wait = int(
        os.environ.get("CP_CLI_PUBLISH_MAX_WAIT_SECONDS", DEFAULT_MAX_WAIT_SECONDS)
    )
    for attempt in range(1, max_attempts + 1):
        print(f"Yanking {package} {version} ({attempt}/{max_attempts})")
        return_code, output = run_cargo(command, workspace)
        if return_code == 0 or "already yanked" in output.lower():
            return
        if not is_rate_limited(output):
            raise ReleaseError(f"cargo yank failed for {package} {version}")
        delay = retry_delay(output)
        if delay > max_wait:
            raise ReleaseError(
                f"crates.io asked to wait {delay}s, above the {max_wait}s safety limit"
            )
        print(f"crates.io rate limit reached; retrying in {delay}s")
        time.sleep(delay)
    raise ReleaseError(f"cargo yank kept rate-limiting {package} {version}")


def self_test() -> None:
    assert sparse_index_path("cp-cli") == "cp/-c/cp-cli"
    assert older_versions(
        {"3.1.10", "3.1.9", "0.9.2", "4.0.0", "preview"}, "3.1.10"
    ) == ["0.9.2", "3.1.9"]
    timestamp = email.utils.parsedate_to_datetime(
        "Tue, 22 Sep 2026 22:10:29 GMT"
    ).timestamp()
    assert retry_delay(
        "Please try again after Tue, 22 Sep 2026 22:10:29 GMT and see docs",
        now=timestamp - 20,
    ) == 25
    assert retry_delay("", retry_after="12") == 13
    assert retry_delay("try again after invalid GMT") == int(
        os.environ.get("CP_CLI_PUBLISH_RETRY_SECONDS", "60")
    )
    assert is_rate_limited("the remote server responded with status 429")
    assert is_rate_limited("HTTP 429 Too Many Requests")
    assert not is_rate_limited("HTTP 422 Unprocessable Entity")
    assert VERSION_QUOTA.search(
        "You have published too many versions of this crate in the last 24 hours"
    )
    quota_now = datetime.fromisoformat("2026-09-23T00:00:00+00:00").timestamp()
    recent = [
        datetime.fromtimestamp(quota_now - 60 * index, tz=timezone.utc).isoformat()
        for index in range(VERSION_QUOTA_LIMIT)
    ]
    assert version_quota_delay(recent, now=quota_now) == 85_266
    for invalid in (["not-a-date"] * VERSION_QUOTA_LIMIT, recent[:-1]):
        try:
            version_quota_delay(invalid, now=quota_now)
        except ReleaseError:
            pass
        else:
            raise AssertionError("invalid quota metadata was accepted")

    attempts: list[str] = []
    waits: list[int] = []

    def deferred_once(_registry, _workspace, package, version, _dry_run):
        attempts.append(package)
        if package == "first" and attempts.count(package) == 1:
            raise PublicationDeferred(package, version, 5)

    publish_queue(
        None,
        Path(),
        [("first", "3.1.10"), ("second", "3.1.10")],
        False,
        publish=deferred_once,
        wait=waits.append,
    )
    assert attempts == ["first", "second", "first"]
    assert waits == [5]

    current = package_version(ROOT / "Cargo.toml")
    with tempfile.TemporaryDirectory(prefix="cp-cli-publish-test-") as temporary:
        workspace = Path(temporary)
        prepare_workspace(workspace)
        assert package_version(workspace / "Cargo.toml") == current
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
    parser.add_argument("--version", help="publish the checked-in release version")
    parser.add_argument(
        "--dry-run", action="store_true", help="run Cargo's checks without uploading"
    )
    parser.add_argument(
        "--yank-all",
        action="store_true",
        help="replace prior releases by publishing current, then yanking every lower version",
    )
    parser.add_argument("--self-test", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    if args.self_test:
        self_test()
        return 0

    current = package_version(ROOT / "Cargo.toml")
    if args.version and args.version != current:
        raise ReleaseError(
            f"requested version {args.version} does not match workspace version {current}"
        )
    registry = Registry(current)

    with tempfile.TemporaryDirectory(prefix="cp-cli-publish-") as temporary:
        workspace = Path(temporary)
        prepare_workspace(workspace)

        platform_releases = [
            (package, current) for package, _manifest in PLATFORM_PACKAGES
        ]
        publish_queue(registry, workspace, platform_releases, args.dry_run)
        if not args.dry_run:
            for package, _manifest in PLATFORM_PACKAGES:
                registry.wait_until_indexed(package, current)
        publish_queue(
            registry, workspace, [("cp-cli", current)], args.dry_run
        )
        if args.yank_all:
            if not args.dry_run:
                registry.wait_until_indexed("cp-cli", current)
            packages = [package for package, _manifest in PLATFORM_PACKAGES]
            packages.append("cp-cli")
            for package in packages:
                for version in older_versions(
                    registry.versions(package, refresh=True), current
                ):
                    yank_package(workspace, package, version, args.dry_run)
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ReleaseError, subprocess.SubprocessError) as error:
        print(f"error: {error}", file=sys.stderr)
        raise SystemExit(1)
