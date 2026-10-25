#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Tests for scripts/check-evidence-staleness.sh, run offline through its EVIDENCE_LATEST_DIR hook.

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

# transcript <dir> <file-name> <install line>
transcript() {
    printf 'custody: ci\n---\ninstall:\n  %s\nexit-codes:\n  install 0\n' "$3" > "$1/$2"
}

# run <evidence-dir> <latest-dir>: sets out and status.
run() {
    set +e
    out=$(EVIDENCE_LATEST_DIR=$2 sh "$root/scripts/check-evidence-staleness.sh" "$1" 2>&1)
    status=$?
    set -e
}

mk() {
    rm -rf "$tmp/ev" "$tmp/latest"
    mkdir -p "$tmp/ev" "$tmp/latest"
}

mk
transcript "$tmp/ev" codex-0.164.0.md "npm install --global @openai/codex"
transcript "$tmp/ev" opencode-v2.1.4.md "npm install --global @opencode/cli"
transcript "$tmp/ev" amp-0.0.1792872041-gb83e0b.md "npm install --global @ampcode/cli"
echo 0.164.0 > "$tmp/latest/codex"
echo 2.1.4 > "$tmp/latest/opencode"
echo 0.0.1792872041-gb83e0b > "$tmp/latest/amp"
run "$tmp/ev" "$tmp/latest"
check "all current: exit 0" "$status" "0"
check "a leading v is ignored and a hyphenated version compares whole" \
    "$(printf '%s\n' "$out" | grep -c '^current')" "3"

echo 0.165.0 > "$tmp/latest/codex"
run "$tmp/ev" "$tmp/latest"
check "one behind: exit 1" "$status" "1"
check "and it is the one named" "$(printf '%s\n' "$out" | grep -c '^BEHIND    codex-0.164.0: latest is 0.165.0')" "1"
check "the others stay current" "$(printf '%s\n' "$out" | grep -c '^current')" "2"

echo 0.0.1792999999-gnewer > "$tmp/latest/amp"
echo 0.165.0 > "$tmp/latest/codex"
run "$tmp/ev" "$tmp/latest"
check "a rolling id that is behind is reported but does not add to the failures" \
    "$(printf '%s\n' "$out" | grep -c '^rolling   amp-')" "1"
check "and the summary counts only codex as behind" \
    "$(printf '%s\n' "$out" | grep -c ' 1 behind, 0 unchecked')" "1"
echo 0.0.1792872041-gb83e0b > "$tmp/latest/amp"

echo 0.164.0 > "$tmp/latest/codex"
: > "$tmp/latest/opencode"
run "$tmp/ev" "$tmp/latest"
check "an empty latest version is unchecked, not current: exit 1" "$status" "1"
check "and says so" "$(printf '%s\n' "$out" | grep -c '^UNCHECKED opencode-v2.1.4')" "1"

echo 2.1.4 > "$tmp/latest/opencode"
rm "$tmp/latest/amp"
run "$tmp/ev" "$tmp/latest"
check "a missing latest version is unchecked: exit 1" "$status" "1"

mk
transcript "$tmp/ev" aider-0.86.2.md "uv tool install --python 3.12 aider-chat"
printf 'custody: off-ci\n---\ninstall:\n  nothing\n' > "$tmp/ev/claude-2.1.310-credentials.md"
echo 0.86.2 > "$tmp/latest/aider"
run "$tmp/ev" "$tmp/latest"
check "a credentials transcript is not compared" "$status" "0"
check "and is not counted" "$(printf '%s\n' "$out" | grep -c 'credentials')" "0"

mk
printf 'custody: ci\n---\nexit-codes:\n  install 0\n' > "$tmp/ev/gemini-0.65.0.md"
run "$tmp/ev" "$tmp/latest"
check "a transcript with no install line is unchecked: exit 1" "$status" "1"

mk
run "$tmp/ev" "$tmp/latest"
check "an empty evidence directory fails rather than reporting clean" "$status" "1"

# A real install line that the script cannot place must fail without the test hook.
mk
transcript "$tmp/ev" mystery-1.0.0.md "gem install mystery"
set +e
out=$(sh "$root/scripts/check-evidence-staleness.sh" "$tmp/ev" 2>&1)
status=$?
set -e
check "an install line it cannot place is an error: exit 1" "$status" "1"
check "and says so" "$(printf '%s\n' "$out" | grep -c '^UNCHECKED mystery-1.0.0')" "1"

[ "$failures" -eq 0 ]
