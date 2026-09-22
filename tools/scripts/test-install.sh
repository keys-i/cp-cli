#!/bin/sh

set -eu

project_dir=$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)
test_dir=$(mktemp -d "${TMPDIR:-/tmp}/cp-cli-install-test.XXXXXX")
trap 'rm -rf "$test_dir"' EXIT HUP INT TERM

package_dir="$test_dir/package"
release_dir="$test_dir/release"
bin_dir="$test_dir/bin"
install_dir="$test_dir/install"
corrupt_install_dir="$test_dir/corrupt-install"
mkdir -p "$package_dir" "$release_dir" "$bin_dir"

cat >"$package_dir/cp-cli" <<'EOF'
#!/bin/sh
test "$1" = "--version"
printf '%s\n' 'cp-cli 3.1.8'
EOF
chmod 755 "$package_dir/cp-cli"
tar -C "$package_dir" -czf "$release_dir/cp-cli-$(uname -m | sed 's/^arm64$/aarch64/; s/^amd64$/x86_64/')-$(test "$(uname -s)" = Darwin && printf apple-darwin || printf unknown-linux-musl).tar.gz" cp-cli

archive=$(find "$release_dir" -name 'cp-cli-*.tar.gz' -exec basename {} \;)
if command -v shasum >/dev/null 2>&1; then
  shasum -a 256 "$release_dir/$archive" | sed "s|$release_dir/|./|" >"$release_dir/SHA256SUMS"
else
  sha256sum "$release_dir/$archive" | sed "s|$release_dir/|./|" >"$release_dir/SHA256SUMS"
fi

cat >"$bin_dir/curl" <<'EOF'
#!/bin/sh
while test "$#" -gt 2; do
  case "$1" in
    --output) output=$2; shift 2 ;;
    *) shift ;;
  esac
done
url=$1
cp "$CP_CLI_TEST_RELEASE/${url##*/}" "$output"
EOF
chmod 755 "$bin_dir/curl"

PATH="$bin_dir:$PATH" CP_CLI_TEST_RELEASE="$release_dir" \
  CP_CLI_DOWNLOAD_BASE="http://127.0.0.1:8080/releases" \
  sh "$project_dir/tools/scripts/install.sh" --version 3.1.8 --install-dir "$install_dir"
test "$("$install_dir/cp-cli" --version)" = "cp-cli 3.1.8"

printf '%064d  *./%s\n' 0 "$archive" >"$release_dir/SHA256SUMS"
if PATH="$bin_dir:$PATH" CP_CLI_TEST_RELEASE="$release_dir" \
  CP_CLI_DOWNLOAD_BASE="http://127.0.0.1:8080/releases" \
  sh "$project_dir/tools/scripts/install.sh" --version 3.1.8 --install-dir "$corrupt_install_dir"; then
  printf '%s\n' 'corrupt checksum was accepted' >&2
  exit 1
fi
test ! -e "$corrupt_install_dir/cp-cli"

for invalid_version in 0.02.0 1.2 1.2.3. 1.2.3-01 vlatest '../1.2.3'; do
  if sh "$project_dir/tools/scripts/install.sh" --version "$invalid_version" --install-dir "$test_dir/invalid-install"; then
    printf '%s\n' "invalid semantic version was accepted: $invalid_version" >&2
    exit 1
  fi
done
printf '%s\n' 'installer self-test passed'
