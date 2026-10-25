#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Fails when a committed evidence transcript is for an older upstream version than the vendor now ships.
#
# Spec V3 §2: "Evidence must be reverified against the supported upstream version before release." Nothing
# compared the two until now, and when it was done by hand nine of twelve transcripts were behind. This
# only DETECTS: refreshing is a manual Sandbox dispatch (probes install real agents), never done here.
#
# WHERE EACH LATEST VERSION COMES FROM is read out of the transcript's own `install:` line, so the check
# cannot disagree with what the probe installed and needs no second list to keep in step:
#   npm install --global [--ignore-scripts] <pkg>   npm view <pkg> version
#   uv tool install ... aider-chat                  PyPI JSON for aider-chat
#   curl ... https://cursor.com/install | bash      the version in that installer's download URL
#   curl ... https://cli.kiro.dev/install | bash    the stable channel's latest manifest.json
# AN INSTALL LINE THIS SCRIPT CANNOT PLACE IS AN ERROR, not a skip: a transcript nobody can check would
# otherwise read as a current one (the same rule as scripts/lib/scan-guard.sh).
#
# ROLLING ids: amp's version is a build timestamp plus a hash and a new one appears continuously (a newer
# build existed hours after the transcript was refreshed), so "behind" there is the permanent state and a
# failing job would be noise nobody reads. It is still printed, as `rolling`, but does not fail the run.
# Add an id here only with that reason; a vendor with ordinary releases must not be listed.
rolling='amp'
#
# `<id>-credentials.md` transcripts are off-CI measurements of a different question and are not compared.
# A version is "current" when it equals the latest after dropping a leading `v`; there is no ordering, so a
# vendor that withdraws a release also shows as behind, which is the safe direction.
#
# Usage: sh scripts/check-evidence-staleness.sh [evidence-dir]   (default: <repo>/docs/evidence)
# Test hook: when EVIDENCE_LATEST_DIR is set, the latest version of <id> is read from the file
#   $EVIDENCE_LATEST_DIR/<id> instead of the network, so the logic is testable offline.
# Output: one line per transcript; exit 1 when any is behind or cannot be checked, else exit 0.

set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
. "$repo_root/scripts/lib/scan-guard.sh"

dir=${1:-$repo_root/docs/evidence}
files=$(mktemp)
trap 'rm -f "$files"' EXIT INT TERM

scan_expand "$files" must-find "evidence transcripts under $dir" "$dir"/*-*.md

install_line() {
    sed -n '/^install:/{n;p;}' "$1" | sed 's/^[[:space:]]*//'
}

# latest <id> <install-line>: prints the latest upstream version, or nothing when it cannot be found.
latest() {
    if [ -n "${EVIDENCE_LATEST_DIR:-}" ]; then
        cat "$EVIDENCE_LATEST_DIR/$1" 2>/dev/null || true
        return 0
    fi
    case "$2" in
        "npm install --global --ignore-scripts "*) npm view "${2##* }" version 2>/dev/null || true ;;
        "npm install --global "*) npm view "${2##* }" version 2>/dev/null || true ;;
        "uv tool install "*"aider-chat")
            curl -fsS https://pypi.org/pypi/aider-chat/json | jq -r '.info.version' || true ;;
        *"cursor.com/install"*)
            curl -fsSL https://cursor.com/install \
                | sed -n 's#.*downloads\.cursor\.com/lab/\([^/]*\)/.*#\1#p' | head -n 1 || true ;;
        *"cli.kiro.dev/install"*)
            curl -fsS https://prod.download.cli.kiro.dev/stable/latest/manifest.json \
                | jq -r '.version' || true ;;
    esac
}

is_rolling() {
    for r in $rolling; do
        [ "$r" = "$1" ] && return 0
    done
    return 1
}

behind=0
unchecked=0
checked=0

while IFS= read -r file; do
    name=$(basename "$file" .md)
    case "$name" in *-credentials) continue ;; esac
    # <id>-<version>: the id has no `-` (adapter ids are single words), the version may.
    id=${name%%-*}
    pinned=${name#*-}
    line=$(install_line "$file")
    if [ -z "$line" ]; then
        echo "UNCHECKED $name: no install line in the transcript"
        unchecked=$((unchecked + 1))
        continue
    fi
    now=$(latest "$id" "$line")
    case "$now" in
        '' | null)
            echo "UNCHECKED $name: could not read the latest version from: $line"
            unchecked=$((unchecked + 1))
            continue
            ;;
    esac
    checked=$((checked + 1))
    if [ "${now#v}" = "${pinned#v}" ]; then
        echo "current   $name"
    elif is_rolling "$id"; then
        echo "rolling   $name: latest is $now"
    else
        echo "BEHIND    $name: latest is $now"
        behind=$((behind + 1))
    fi
done < "$files"

echo "check-evidence-staleness: $checked compared, $behind behind, $unchecked unchecked"
[ "$behind" -eq 0 ] && [ "$unchecked" -eq 0 ]
