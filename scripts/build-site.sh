#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Usage: build-site.sh <version> <out-dir>
#
# Copies site/ into <out-dir> and fills the three placeholders: {{VERSION}} with <version>, and
# {{DEMO_VERSION}} / {{DEMO_DATE}} with the version= / date= lines of site/demo/agents.meta. It then writes
# <out-dir>/agent_profile/index.html and <out-dir>/fake_agent/index.html, redirects to the same page under
# rustdoc/, so the rustdoc links that were valid before the site moved keep working.
#
# Offline on purpose: the version is an argument, not a lookup. .github/workflows/docs.yml reads it from the
# latest published release; a local preview passes any string. Exits 2 on a usage error.

set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: build-site.sh <version> <out-dir>" >&2
    exit 2
fi
version=$1
out=$2
root=$(cd "$(dirname "$0")/.." && pwd)
src=$root/site

# Spliced into sed below, so held to the charset a version and a date can have.
safe() {
    case $2 in
        '' | *[!A-Za-z0-9._+-]*)
            echo "build-site.sh: $1 must match [A-Za-z0-9._+-]+, got '$2'" >&2
            exit 2
            ;;
    esac
}
safe version "$version"
demo_version=$(sed -n 's/^version=//p' "$src/demo/agents.meta")
demo_date=$(sed -n 's/^date=//p' "$src/demo/agents.meta")
safe "demo version in agents.meta" "$demo_version"
safe "demo date in agents.meta" "$demo_date"

mkdir -p "$out"
cp -R "$src/." "$out/"
sed -e "s/{{VERSION}}/$version/g" \
    -e "s/{{DEMO_VERSION}}/$demo_version/g" \
    -e "s/{{DEMO_DATE}}/$demo_date/g" \
    "$src/index.html" >"$out/index.html"

for crate in agent_profile fake_agent; do
    mkdir -p "$out/$crate"
    cat >"$out/$crate/index.html" <<STUB
<!doctype html>
<meta charset="utf-8">
<title>Moved</title>
<meta http-equiv="refresh" content="0; url=../rustdoc/$crate/index.html">
<link rel="canonical" href="../rustdoc/$crate/index.html">
<p>The API documentation moved to <a href="../rustdoc/$crate/index.html">rustdoc/$crate/</a>.</p>
STUB
done
