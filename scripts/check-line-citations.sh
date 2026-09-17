#!/bin/sh
# Fails when a COMMENT in live shell, Rust or workflow source cites another file by LINE NUMBER -- e.g.
# `sandbox/run.sh:192` or `aider.rs:17-18` (line-cite-ok -- these two are this gate's own illustrations of
# the defect, not claims about those files). A line number rots the moment EITHER file is edited, including
# by someone who never touches the citing comment: three separate commits in one day staled thirteen such
# citations across this tree, and two more were already pointing at the wrong place before that, silently,
# because nothing checked them. Cite the SYMBOL, the function, or a short quoted phrase from the target
# instead -- something that does not move when the file around it does.
#
# Usage:
#   sh scripts/check-line-citations.sh
#
# Scope: COMMENT lines in sandbox/, crates/, .github/ and scripts/ -- *.sh, *.rs and *.yml. Only comment
# lines are scanned because a `path:line` inside CODE is normally test data rather than a citation:
# sandbox/tests/probe-harness.sh stages a python traceback of exactly that shape, and silencing it with the
# escape hatch below would be the wrong use of an escape hatch. docs/ is a historical record of the code as
# it stood and is deliberately NOT scanned -- rewriting its citations would falsify that record.
#
# Escape hatch, for the rare citation that genuinely cannot be expressed any other way:
#   - inline:   add the literal marker `line-cite-ok` anywhere on the same line as the citation
#   - allowlist: list the citing `path:line` (repo-relative, one per line; blank lines and `#` comments
#                are ignored) in scripts/line-citation-allowlist.txt
set -eu

repo_root=$(CDPATH='' cd -- "$(dirname -- "$0")/.." && pwd)
# The repository's one answer to "a scan that produces nothing is read as clean" -- the class this gate
# was the second measured instance of. The refusal below used to be spelled out by hand here.
. "$repo_root/scripts/lib/scan-guard.sh"
allowlist="$repo_root/scripts/line-citation-allowlist.txt"
marker='line-cite-ok'

# A comment line (`#`, `//` or `///`) that names a file and then a line number. The extension list is
# deliberately the set of things this repository actually cites; `.py` is absent on purpose, because the
# only `.py:NN` strings here are staged traceback fixtures. Containerfile, justfile and Makefile carry no
# extension and are matched by name.
pattern='^[[:space:]]*(#|//).*([a-zA-Z0-9_/.-]+\.(sh|rs|yml|yaml|toml|md|txt|json)|Containerfile|justfile|Makefile):[0-9]+'

usage() {
    echo "usage: $0" >&2
    exit 2
}
[ $# -eq 0 ] || usage

files=$(mktemp)
hits=$(mktemp)
trap 'rm -f "$files" "$hits"' EXIT

# `find` is its own statement, NOT the left-hand side of a pipe. As the left-hand side its exit status was
# discarded -- POSIX sh has no pipefail -- and the `|| true` that grep needs for "no matches" swallowed the
# pipeline's status as well, so a renamed root, a partial checkout or a worktree left $hits empty, the loop
# below ran zero times, and this gate certified a tree it had never read. That is the same false negative
# b140a18 fixed in scripts/gen-tool-table.sh, where jq was first in a pipeline.
#
# find, not a glob: an unmatched glob passes through literally and would hand grep a filename that does not
# exist (same reasoning as the `shellcheck` recipe in the justfile).
#
# `-print`, not `-print0`, now that the hits are collected one file at a time below: POSIX sh cannot read a
# NUL-delimited list at all (`read` has no -d). A path containing a newline splits into two lines here, and
# neither half names a file, so grep exits 2 and this gate REFUSES -- which is the direction the whole file
# is about, and the opposite of skipping it in silence.
find "$repo_root/sandbox" "$repo_root/crates" "$repo_root/.github" "$repo_root/scripts" \
    \( -name '*.sh' -o -name '*.rs' -o -name '*.yml' \) -type f -print > "$files"

# Any ONE of those roots may legitimately hold no matching file; all four holding none cannot happen in a
# healthy checkout, so an empty list is a broken scan and not a clean tree. Refusing here is what keeps a
# zero-failure result meaningful. That reasoning is now the shared `must-find` policy in scan-guard.sh.
scan_require "$files" ".sh, .rs or .yml files under sandbox/, crates/, .github/ or scripts/" || exit 1

# ONE GREP PER FILE, through scan_grep, and that is the fix rather than the tidying it looks like. A single
# `xargs -0 grep` cannot report a hard error: xargs collapses "some invocation exited 1-125" into its own
# status 123, and grep exits 1 for "no match" and 2 for "could not read that file", so the two reach the
# caller as one indistinguishable number -- which `|| true` then discarded anyway. Per file, grep's own
# status survives and scan_grep refuses anything above 1. Roughly thirty files; the cost is not measurable
# beside the find.
#
# /dev/null forces grep to prefix every match with its filename even though only one real file is passed to
# a given invocation, so the `path:line:content` shape the loop below parses is unchanged.
scan_broken=0
while IFS= read -r scanned; do
    [ -n "$scanned" ] || continue
    if ! scan_grep -nE "$pattern" /dev/null "$scanned" >> "$hits"; then
        scan_broken=1
    fi
done < "$files"

[ "$scan_broken" -eq 0 ] || {
    echo "check-line-citations: part of the tree could not be read, so this scan is broken, not clean." >&2
    exit 1
}

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
    echo "check-line-citations: $failures line-pinned citation(s) in a scanned comment." >&2
    echo "A line number rots the moment the cited file changes elsewhere. Cite the symbol, the function," >&2
    echo "or a short quoted phrase from the target instead of the line it currently happens to sit on." >&2
    exit 1
fi

echo "check-line-citations: no line-pinned citations found"
