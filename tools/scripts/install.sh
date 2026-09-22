#!/bin/sh

set -eu

repository_base="https://github.com/keys-i/cp-cli/releases"
download_base="${CP_CLI_DOWNLOAD_BASE:-$repository_base}"
release_version="latest"
requested_version=""

usage() {
  cat <<'EOF'
Install cp-cli from its GitHub Release.

Usage: install.sh [--version VERSION] [--install-dir DIRECTORY]

Options:
  --version VERSION       Install VERSION (for example 1.2.3); defaults to latest
  --install-dir DIRECTORY Install the binary in DIRECTORY; defaults to ~/.local/bin
  -h, --help              Show this help

Set CP_CLI_DOWNLOAD_BASE to an HTTPS release base for a mirror, or an HTTP
localhost/127.0.0.1 base for local testing.
EOF
}

fail() {
  printf '%s\n' "error: $*" >&2
  exit 1
}

is_numeric_identifier() {
  case "$1" in
    ""|*[!0-9]*) return 1 ;;
    0) return 0 ;;
    0*) return 1 ;;
    *) return 0 ;;
  esac
}

is_semver_identifier() {
  case "$1" in
    ""|*[!0-9A-Za-z-]*) return 1 ;;
  esac
}

is_identifier_list() {
  identifier_list=$1
  reject_leading_zero=$2
  case "$identifier_list" in ""|.*|*.|*..*) return 1 ;; esac
  while :; do
    case "$identifier_list" in
      *.*)
        identifier=${identifier_list%%.*}
        identifier_list=${identifier_list#*.}
        ;;
      *)
        identifier=$identifier_list
        identifier_list=""
        ;;
    esac
    is_semver_identifier "$identifier" || return 1
    if test "$reject_leading_zero" = true; then
      case "$identifier" in
        *[!0-9]*) ;;
        *) is_numeric_identifier "$identifier" || return 1 ;;
      esac
    fi
    test -n "$identifier_list" || break
  done
}

is_semver() {
  semver_value=$1
  case "$semver_value" in ""|*[!0-9A-Za-z.+-]*) return 1 ;; esac

  core_and_pre=${semver_value%%+*}
  if test "$core_and_pre" != "$semver_value"; then
    build=${semver_value#*+}
    case "$build" in ""|*+*) return 1 ;; esac
    is_identifier_list "$build" false || return 1
  fi

  core=${core_and_pre%%-*}
  if test "$core" != "$core_and_pre"; then
    prerelease=${core_and_pre#*-}
    is_identifier_list "$prerelease" true || return 1
  fi

  major=${core%%.*}
  remaining=${core#*.}
  test "$remaining" != "$core" || return 1
  minor=${remaining%%.*}
  patch=${remaining#*.}
  test "$patch" != "$remaining" || return 1
  case "$patch" in *.*) return 1 ;; esac
  is_numeric_identifier "$major" && is_numeric_identifier "$minor" && is_numeric_identifier "$patch"
}

while test "$#" -gt 0; do
  case "$1" in
    --version)
      test "$#" -ge 2 || fail "--version needs a value"
      release_version="$2"
      requested_version="$2"
      shift 2
      ;;
    --install-dir)
      test "$#" -ge 2 || fail "--install-dir needs a directory"
      install_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "unknown option: $1"
      ;;
  esac
done

: "${HOME:?HOME must be set}"
install_dir="${install_dir:-${XDG_BIN_HOME:-$HOME/.local/bin}}"

case "$download_base" in
  https://*|http://localhost|http://localhost/*|http://localhost:*|http://127.0.0.1|http://127.0.0.1/*|http://127.0.0.1:*)
    ;;
  *)
    fail "CP_CLI_DOWNLOAD_BASE must use HTTPS, except for local HTTP testing"
    ;;
esac
download_base=${download_base%/}

case "$release_version" in
  latest)
    release_path="latest/download"
    ;;
  v*)
    release_version=${release_version#v}
    is_semver "$release_version" || fail "version must be valid semantic versioning"
    release_path="download/v$release_version"
    ;;
  *)
    is_semver "$release_version" || fail "version must be valid semantic versioning"
    release_path="download/v$release_version"
    ;;
esac

case "$(uname -s)" in
  Darwin)
    case "$(uname -m)" in
      arm64|aarch64) target="aarch64-apple-darwin" ;;
      x86_64|amd64) target="x86_64-apple-darwin" ;;
      *) fail "unsupported macOS architecture: $(uname -m)" ;;
    esac
    ;;
  Linux)
    case "$(uname -m)" in
      aarch64|arm64) target="aarch64-unknown-linux-musl" ;;
      x86_64|amd64) target="x86_64-unknown-linux-musl" ;;
      *) fail "unsupported Linux architecture: $(uname -m)" ;;
    esac
    ;;
  *)
    fail "unsupported operating system: $(uname -s)"
    ;;
esac

archive="cp-cli-$target.tar.gz"
checksum_file="SHA256SUMS"
release_url="$download_base/$release_path"
temporary_dir=$(mktemp -d "${TMPDIR:-/tmp}/cp-cli-install.XXXXXX") || fail "could not create a temporary directory"
trap 'rm -rf "$temporary_dir"' EXIT HUP INT TERM

download() {
  destination=$1
  url=$2
  if command -v curl >/dev/null 2>&1; then
    case "$download_base" in
      http://*) curl --fail --silent --show-error --output "$destination" "$url" ;;
      *) curl --fail --location --proto '=https' --proto-redir '=https' --silent --show-error --output "$destination" "$url" ;;
    esac
  elif command -v wget >/dev/null 2>&1; then
    case "$download_base" in
      http://*) wget --quiet --output-document="$destination" "$url" ;;
      *) wget --https-only --quiet --output-document="$destination" "$url" ;;
    esac
  else
    fail "curl or wget is required to download cp-cli"
  fi
}

sha256() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v openssl >/dev/null 2>&1; then
    openssl dgst -sha256 "$1" | awk '{print $NF}'
  else
    fail "shasum, sha256sum, or openssl is required to verify cp-cli"
  fi
}

printf 'Installing cp-cli for %s…\n' "$target"
download "$temporary_dir/$archive" "$release_url/$archive" || fail "could not download $archive"
download "$temporary_dir/$checksum_file" "$release_url/$checksum_file" || fail "could not download $checksum_file"

expected_hash=$(awk -v archive="$archive" '
  {
    filename = $2
    sub(/^\*/, "", filename)
    sub(/^\.\//, "", filename)
    if (filename == archive) { print $1; exit }
  }
' "$temporary_dir/$checksum_file")
case "$expected_hash" in
  *[!0-9A-Fa-f]*|"") fail "no valid SHA-256 checksum found for $archive" ;;
esac
test "${#expected_hash}" -eq 64 || fail "no valid SHA-256 checksum found for $archive"
expected_hash=$(printf '%s' "$expected_hash" | tr '[:upper:]' '[:lower:]')
actual_hash=$(sha256 "$temporary_dir/$archive" | tr '[:upper:]' '[:lower:]')
test "$actual_hash" = "$expected_hash" || fail "checksum verification failed for $archive"

unpack_dir="$temporary_dir/unpack"
mkdir "$unpack_dir"
tar -xzf "$temporary_dir/$archive" -C "$unpack_dir" || fail "could not unpack $archive"
test -f "$unpack_dir/cp-cli" || fail "$archive does not contain cp-cli"
chmod 755 "$unpack_dir/cp-cli"
installed_version=$("$unpack_dir/cp-cli" --version 2>&1) || fail "downloaded cp-cli could not run"
if test -n "$requested_version"; then
  test "$installed_version" = "cp-cli $release_version" || fail "downloaded cp-cli reported '$installed_version', expected cp-cli $release_version"
fi

mkdir -p "$install_dir" || fail "could not create $install_dir"
install_tmp=$(mktemp "$install_dir/.cp-cli.XXXXXX") || fail "could not prepare installation"
cp "$unpack_dir/cp-cli" "$install_tmp" || fail "could not install cp-cli"
chmod 755 "$install_tmp"
mv -f "$install_tmp" "$install_dir/cp-cli"

printf 'Installed %s to %s\n' "$installed_version" "$install_dir/cp-cli"
case ":$PATH:" in
  *":$install_dir:"*) ;;
  *) printf 'Add %s to PATH to run cp-cli from any directory.\n' "$install_dir" ;;
esac
