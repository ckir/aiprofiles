#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Usage: release-finalize.sh <tag> <directory>
#
# Writes <file>.sha256 ("<hex>  <file>\n", the sha256sum format) beside every agent-profile-*.tar.gz and
# agent-profile-*.zip in <directory>, overwriting any earlier one. Other files are ignored. <tag> is only
# echoed. Exits 1 when no archive matches (a scan that found nothing must not look like success) and 2 on
# a usage error. Needs GNU sha256sum, which the release job's ubuntu runner has.

set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: release-finalize.sh <tag> <directory>" >&2
    exit 2
fi
tag=$1
dir=$2

list=$(find "$dir" -maxdepth 1 -type f \( -name 'agent-profile-*.tar.gz' -o -name 'agent-profile-*.zip' \) | sort)
if [ -z "$list" ]; then
    echo "release-finalize.sh: no agent-profile archive in $dir" >&2
    exit 1
fi

count=0
for path in $list; do
    name=$(basename "$path")
    (cd "$dir" && sha256sum "$name" >"$name.sha256")
    count=$((count + 1))
done
echo "release-finalize.sh: wrote $count checksum file(s) for $tag in $dir"
