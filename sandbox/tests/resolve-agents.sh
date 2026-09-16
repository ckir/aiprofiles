#!/bin/sh
# Tests for sandbox/resolve-agents.sh, the Sandbox workflow's matrix resolution.
#
# Every rule it enforces is one that fails QUIETLY when it is missing: an empty input makes a
# split-and-loop iterate zero times and report success having probed nothing, and a duplicated name
# doubles the runner cost while colliding on the artifact name the matrix keeps unique, so the second
# upload overwrites the first. None of that turns a job red on its own.

set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
failures=0

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

resolve() {
    set +e
    out=$(sh "$root/sandbox/resolve-agents.sh" "$@" 2>/dev/null)
    status=$?
    set -e
}

resolve claude
check "one agent resolves to a one-element array" "$out" '["claude"]'
check "one agent succeeds" "$status" "0"

resolve "claude codex aider"
check "whitespace separates names" "$out" '["claude","codex","aider"]'

resolve "claude,codex"
check "commas separate names" "$out" '["claude","codex"]'

resolve "claude, codex,  aider"
check "commas and spaces mix" "$out" '["claude","codex","aider"]'

resolve "codex claude codex"
check "a repeated name appears once, in the order given" "$out" '["codex","claude"]'

resolve all
check "all resolves to every probe script" "$out" \
    '["aider","amp","claude","cline","codex","continue","copilot","cursor","gemini","kiro","opencode","pi"]'

# The library files live in the same directory. Naming them by hand and excluding them by hand would be a
# list that rots; `all` is defined by the property of sourcing the harness instead.
check "all excludes the harness itself" "$(printf '%s' "$out" | grep -c common)" "0"
check "all excludes the text helpers" "$(printf '%s' "$out" | grep -c text)" "0"

resolve ""
check "an empty input is refused" "$status" "1"

resolve "   "
check "whitespace alone is refused" "$status" "1"

resolve "nosuchagent"
check "an unknown agent is refused" "$status" "1"

resolve "common"
check "a library file is not an agent" "$status" "1"

resolve "Claude"
check "a name outside the charset is refused" "$status" "1"

resolve "../../etc/passwd"
check "a path is refused" "$status" "1"

resolve "a b c d e f g h i j k l m n o p q r s t u v w x y"
check "more than the cap is refused" "$status" "1"

resolve claude 2.1.270
check "a version with one agent is accepted" "$status" "0"

# A pinned version belongs to one package. Applied across a matrix it would ask npm for one agent at
# another's version number, and the refusal would be recorded as a failed install.
resolve "claude codex" 2.1.270
check "a version with several agents is refused" "$status" "1"

# The agent list is validated one name at a time, and then BOTH values are written to $GITHUB_OUTPUT,
# which is a key=value-per-line file. A version containing a newline writes a second `agents=` line that
# overrides the validated one, so the list reaching the matrix would have passed no check at all.
resolve claude 'x
agents=["evil"]'
check "a version carrying a newline is refused" "$status" "1"

resolve claude 'v1.0 --flag'
check "a version outside Gate A's charset is refused" "$status" "1"

resolve claude 1.2.3-beta.1
check "a version using every permitted character is accepted" "$status" "0"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all resolve-agents checks passed"
