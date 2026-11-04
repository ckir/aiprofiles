#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Tests for scripts/release-finalize.sh, which writes a sha256sum-format .sha256 beside every release
# archive. An empty directory must fail rather than report success (the scan-guard class).

set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
failures=0
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT INT TERM

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

dir="$tmp/assets"
mkdir "$dir"
for t in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu x86_64-apple-darwin aarch64-apple-darwin; do
    printf 'bytes of %s\n' "$t" >"$dir/agent-profile-$t.tar.gz"
done
printf 'bytes of windows\n' >"$dir/agent-profile-x86_64-pc-windows-msvc.zip"
printf 'stale' >"$dir/agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256"
printf 'readme\n' >"$dir/README.txt"

set +e
out=$(sh "$root/scripts/release-finalize.sh" v9.9.9 "$dir" 2>&1)
status=$?
set -e
check "t1 exit status" "$status" "0"

count=$(find "$dir" -name '*.sha256' | wc -l | tr -d ' ')
check "t2 five .sha256 files" "$count" "5"

zip=agent-profile-x86_64-pc-windows-msvc.zip
want="$(cd "$dir" && sha256sum "$zip" | cut -d' ' -f1)  $zip"
got=$(cat "$dir/$zip.sha256")
check "t3 windows checksum content" "$got" "$want"

stale=$(cat "$dir/agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256")
was_stale=no
[ "$stale" = "stale" ] && was_stale=yes
check "t4 stale checksum overwritten" "$was_stale" "no"

extra=0
[ -e "$dir/README.txt.sha256" ] && extra=1
[ -e "$dir/agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256.sha256" ] && extra=1
check "t5 no checksum of non-archives or of checksums" "$extra" "0"

empty="$tmp/empty"
mkdir "$empty"
set +e
out=$(sh "$root/scripts/release-finalize.sh" v9.9.9 "$empty" 2>&1)
status=$?
set -e
check "t6 empty directory exits 1" "$status" "1"
case $out in *"$empty"*) check "t6 message names the directory" "yes" "yes" ;; *) check "t6 message names the directory" "no" "yes" ;; esac

set +e
sh "$root/scripts/release-finalize.sh" v9.9.9 >/dev/null 2>&1
status=$?
set -e
check "t7 one argument exits 2" "$status" "2"

if [ "$failures" -ne 0 ]; then
    echo "$failures failure(s)"
    exit 1
fi
