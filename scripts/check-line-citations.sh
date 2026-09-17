#!/bin/sh
# Fails when a comment in live shell or Rust source cites another file by LINE NUMBER -- `sandbox/run.sh:192`,
# `aider.rs:17-18`. A line number rots the moment EITHER file is edited, including by someone who never
# touches the citing comment: three separate commits in one day staled thirteen such citations across this
# tree, and two more were already pointing at the wrong place before that, silently, because nothing checked
# them. Cite the SYMBOL, the function, or a short quoted phrase from the target instead -- something that
# does not move when the file around it does.
#
# Usage:
#   sh scripts/check-line-citations.sh
#
# Scope: sandbox/**/*.sh and crates/**/*.rs only. docs/ is a historical record of the code as it stood and
# is deliberately not scanned -- rewriting its citations would falsify that record.
#
# Escape hatch, for the rare citation that genuinely cannot be expressed any other way:
#   - inline:   add the literal marker `line-cite-ok` anywhere on the same line as the citation
#   - allowlist: list the citing `path:line` (repo-relative, one per line; blank lines and `#` comments
#                are ignored) in scripts/line-citation-allowlist.txt
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
allowlist="$repo_root/scripts/line-citation-allowlist.txt"
marker='line-cite-ok'

usage() {
    echo "usage: $0" >&2
    exit 2
}
[ $# -eq 0 ] || usage

hits=$(mktemp)
trap 'rm -f "$hits"' EXIT

# find, not a glob: an unmatched glob passes through literally and would hand grep a filename that does
# not exist (same reasoning as the `shellcheck` recipe in the justfile). /dev/null forces grep to prefix
# every match with its filename even when only one real file is passed to a given invocation.
find "$repo_root/sandbox" "$repo_root/crates" \( -name '*.sh' -o -name '*.rs' \) -type f -print0 \
    | xargs -0 grep -nE '[a-zA-Z0-9_/.-]+\.(sh|rs|yml|toml):[0-9]+' /dev/null > "$hits" || true

failures=0
while IFS= read -r line; do
    [ -n "$line" ] || continue
    file=${line%%:*}
    rest=${line#*:}
    lineno=${rest%%:*}
    content=${rest#*:}

    case "$content" in
        *"$marker"*) continue ;;
    esac

    rel=${file#"$repo_root"/}
    if [ -f "$allowlist" ] && grep -qxF "$rel:$lineno" "$allowlist"; then
        continue
    fi

    echo "check-line-citations: $rel:$lineno cites another file by line number:" >&2
    echo "  $content" >&2
    failures=$((failures + 1))
done < "$hits"

if [ "$failures" -ne 0 ]; then
    echo >&2
    echo "check-line-citations: $failures line-pinned citation(s) in sandbox/**/*.sh or crates/**/*.rs." >&2
    echo "A line number rots the moment the cited file changes elsewhere. Cite the symbol, the function," >&2
    echo "or a short quoted phrase from the target instead of the line it currently happens to sit on." >&2
    exit 1
fi

echo "check-line-citations: no line-pinned citations found"
