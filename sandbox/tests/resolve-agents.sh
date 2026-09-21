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

# CAP=24 in sandbox/resolve-agents.sh, and the test above passes 25 names -- the boundary itself, 24
# accepted and 25 refused, was never touched. Every real name also has to pass the per-agent loop's own
# `grep` check, "is this a genuine probe script", and the repository has only twelve of those -- fewer
# than the cap. So this builds an ISOLATED COPY of resolve-agents.sh, with its own
# scripts/lib/scan-guard.sh and 25 throwaway one-line probe scripts under a temporary sandbox/probes/,
# and runs THAT copy. `root=$(cd "$(dirname "$0")/.." && pwd)` in resolve-agents.sh resolves relative to
# wherever the script itself lives, so copying it to <tmp>/sandbox/resolve-agents.sh alongside
# <tmp>/sandbox/probes/*.sh makes the cap boundary exercisable without touching the real registry or the
# real cap.
cap_root=$(mktemp -d)
mkdir -p "$cap_root/sandbox/probes" "$cap_root/scripts/lib"
cp "$root/sandbox/resolve-agents.sh" "$cap_root/sandbox/resolve-agents.sh"
cp "$root/scripts/lib/scan-guard.sh" "$cap_root/scripts/lib/scan-guard.sh"
cap_names=
cap_i=1
while [ "$cap_i" -le 25 ]; do
    echo '. sandbox/probes/common.sh' > "$cap_root/sandbox/probes/agent$cap_i.sh"
    cap_names="$cap_names agent$cap_i"
    cap_i=$((cap_i + 1))
done
cap_names=${cap_names# }
cap24=$(printf '%s\n' "$cap_names" | tr ' ' '\n' | sed -n '1,24p' | tr '\n' ' ')
cap24=${cap24% }

set +e
cap24_out=$(sh "$cap_root/sandbox/resolve-agents.sh" "$cap24" 2>/dev/null)
cap24_status=$?
set -e
check "exactly the cap is accepted" "$cap24_status" "0"
check "and the accepted array holds all 24 names" \
    "$(printf '%s' "$cap24_out" | grep -o '"[^"]*"' | wc -l | tr -d ' ')" "24"

# The resolver also requires every name to be a real probe script, checked one at a time AFTER the cap.
# A refusal here could in principle come from either check, so this asserts the specific MESSAGE the cap
# refusal prints -- naming the count and the cap -- rather than trusting exit status 1 alone to mean the
# boundary was what fired.
set +e
cap25_err=$(sh "$cap_root/sandbox/resolve-agents.sh" "$cap_names" 2>&1 >/dev/null)
cap25_status=$?
set -e
check "one more than the cap is refused" "$cap25_status" "1"
check "and the refusal names the cap" "$cap25_err" "resolve-agents: 25 agents requested; the cap is 24"

rm -rf "$cap_root"

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
