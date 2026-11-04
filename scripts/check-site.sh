#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Usage: check-site.sh [<site-dir>]      (default: site)
#
# Gates the landing page against the things it must agree with:
#   - the release matrix in .github/workflows/release.yml (every archive the page names exists, every
#     target has a tab),
#   - latest-release downloads only, no host beyond github.com, polyformproject.org and the page's own
#     origin, no external script or stylesheet,
#   - the committed demo capture site/demo/agents.txt (the page shows an unedited prefix of it),
#   - snippets that verify the checksum before they unpack or run anything.
# Prints one `check-site: <rule>: <detail>` line per failure and exits 1; exits 0 and prints nothing when
# every rule holds. Offline: it reads files only.

set -eu
root=$(cd "$(dirname "$0")/.." && pwd)
site=${1:-$root/site}
page=$site/index.html
failures=0

fail() {
    echo "check-site: $1: $2"
    failures=$((failures + 1))
}

if [ ! -f "$page" ]; then
    fail "page" "$page does not exist"
    exit 1
fi
if [ ! -f "$site/demo/agents.txt" ]; then
    fail "demo" "$site/demo/agents.txt does not exist"
    exit 1
fi

# The matrix is the oracle: `target:` is followed by its `archive:`.
pairs=$(awk '/^ +target: /{t=$2} /^ +archive: /{print t " " $2}' "$root/.github/workflows/release.yml")
npairs=$(printf '%s\n' "$pairs" | grep -c .)
if [ "$npairs" -ne 5 ]; then
    fail "release.yml matrix" "expected 5 targets, found $npairs"
    exit 1
fi

assets=
while read -r target archive; do
    assets="$assets agent-profile-$target.$archive"
    if ! snippet=$(sh "$root/scripts/extract-snippet.sh" "$page" "$target" 2>/dev/null); then
        fail "missing snippet" "$target"
        continue
    fi
    # Without JS every block is visible; a hidden attribute in the source would hide it.
    if grep "id=\"snip-$target\"" "$page" | grep -q ' hidden'; then
        fail "snippet hidden without JS" "$target"
    fi
    # extract-snippet.sh skips the whole opening line, so the tag must be all that is on it.
    if ! grep "id=\"snip-$target\"" "$page" | grep -qE '^<pre [^>]*>$'; then
        fail "text on the opening <pre> line" "$target"
    fi
    # With scripts on, a tab with no button leaves its panel unreachable.
    if ! grep -q "id=\"tab-$target\"" "$page"; then
        fail "missing tab" "$target"
    fi
    # A tab may only download and unpack its own asset.
    for name in $(printf '%s\n' "$snippet" | grep -oE 'agent-profile-[A-Za-z0-9_-]+\.(tar\.gz|zip)' | sort -u); do
        case $name in
            "agent-profile-$target."*) ;;
            *) fail "wrong asset" "$target names $name" ;;
        esac
    done
    case $target in
        *linux*) tool='sha256sum -c -' label="unverified unpack" ;;
        *apple-darwin) tool='shasum -a 256 -c -' label="unverified unpack (macOS)" ;;
        *) tool= ;;
    esac
    if [ -n "$tool" ]; then
        case $snippet in
            *'tar -xzf'*) ;;
            *) fail "no unpack step" "$target" ;;
        esac
        before=${snippet%%tar -xzf*}
        case $before in
            *"$tool"*)
                # What follows the checksum command up to the unpack must be `&&` and nothing that lets
                # the chain carry on after a mismatch.
                between=${before#*"$tool"}
                case $between in
                    *'&&'*) ;;
                    *) fail "$label" "$target" ;;
                esac
                case $between in
                    *';'* | *'||'*) fail "$label" "$target" ;;
                esac
                ;;
            *) fail "$label" "$target" ;;
        esac
    else
        case $snippet in
            *"\$ErrorActionPreference = 'Stop'"*) ;;
            *) fail "unguarded PowerShell" "$target" ;;
        esac
        before=${snippet%%Expand-Archive*}
        case $before in
            *Get-FileHash*' -and '*-ieq* | *Get-FileHash*' -and '*-eq*) ;;
            *) fail "unverified unpack (Windows)" "$target" ;;
        esac
    fi
done <<EOF2
$pairs
EOF2

# Every archive the page names is one the release builds.
# shellcheck disable=SC2013 # the pattern only matches [A-Za-z0-9_.-] tokens, so word splitting is exact
for name in $(grep -oE 'agent-profile-[A-Za-z0-9_-]+\.(tar\.gz|zip)' "$page" | sort -u); do
    case " $assets " in
        *" $name "*) ;;
        *) fail "asset not in release.yml" "$name" ;;
    esac
done

if grep -q 'releases/download/' "$page"; then
    fail "not releases/latest/download" "$(grep -o '[^ "]*releases/download/[^ "]*' "$page" | head -1)"
fi

# The privacy guard covers every file the page loads, not only the HTML: a stylesheet `@import` is as much a
# request to another origin as a <link>.
sources=
for f in "$site"/*.html "$site"/*.css "$site"/*.js; do
    [ -f "$f" ] && sources="$sources $f"
done
for f in $sources; do
    # shellcheck disable=SC2013 # the pattern only matches [A-Za-z0-9._-] tokens, so word splitting is exact
    for host in $(grep -oiE 'https?://[A-Za-z0-9._-]+' "$f" | sed 's#^[Hh][Tt][Tt][Pp][Ss]\{0,1\}://##' | tr '[:upper:]' '[:lower:]' | sort -u); do
        case $host in
            github.com | polyformproject.org | ckir.github.io) ;;
            *) fail "external host" "$host in $(basename "$f")" ;;
        esac
    done
    if grep -qiE "(src|href)=[\"']//|url\(['\"]?//|@import[^;]*['\"(]//" "$f"; then
        fail "external host" "protocol-relative URL in $(basename "$f")"
    fi
done

# The demo is the first N lines of the capture (N stops before the blank line that precedes the fourth
# `agent:`) plus the one line the page adds.
fourth=$(grep -n '^agent:' "$site/demo/agents.txt" | sed -n 4p | cut -d: -f1)
if [ -z "$fourth" ]; then
    fail "demo" "capture has fewer than 4 agent: blocks"
else
    want=$(
        head -n "$((fourth - 2))" "$site/demo/agents.txt"
        printf '… and 9 more\n'
    )
    got=$(sh "$root/scripts/extract-snippet.sh" "$page" demo 2>/dev/null || true)
    if [ "$got" != "$want" ]; then
        fail "demo not a prefix" "the <pre id=\"demo\"> text differs from the first lines of demo/agents.txt"
    fi
fi

for placeholder in '{{VERSION}}' '{{DEMO_VERSION}}' '{{DEMO_DATE}}'; do
    count=$(grep -oF "$placeholder" "$page" | wc -l | tr -d ' ')
    if [ "$count" -ne 1 ]; then
        fail "placeholder" "$placeholder appears $count times, expected once"
    fi
done

[ "$failures" -eq 0 ] || exit 1
