#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Usage: extract-snippet.sh <html-file> <target>
#
# Prints the text of the install snippet <pre class="snippet" id="snip-<target>"> (or, for the target
# `demo`, of <pre id="demo">): tags stripped, &lt; &gt; &quot; &#39; &amp; decoded. The page keeps each
# block's opening tag and its </pre> on lines of their own, so a line range is enough. The snippet-timing
# workflow runs this output verbatim, so what is timed is what the page shows.
# Exits 1 when the block is absent and 2 on a usage error.

set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: extract-snippet.sh <html-file> <target>" >&2
    exit 2
fi
html=$1
target=$2
case $target in
    '' | *[!A-Za-z0-9_-]*)
        echo "extract-snippet.sh: target must match [A-Za-z0-9_-]+" >&2
        exit 2
        ;;
esac
if [ "$target" = demo ]; then
    open='<pre id="demo"'
else
    open="<pre class=\"snippet\" id=\"snip-$target\""
fi

if ! grep -qF -- "$open" "$html"; then
    echo "extract-snippet.sh: no block $open in $html" >&2
    exit 1
fi

awk -v open="$open" '
    index($0, open) == 1 { on = 1; next }
    on && /^<\/pre>/ { exit }
    on { print }
' "$html" | sed \
    -e 's/<[^>]*>//g' \
    -e 's/&lt;/</g' \
    -e 's/&gt;/>/g' \
    -e 's/&quot;/"/g' \
    -e "s/&#39;/'/g" \
    -e 's/&amp;/\&/g'
