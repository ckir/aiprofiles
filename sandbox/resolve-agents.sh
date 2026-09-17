#!/bin/sh
# Resolves the Sandbox workflow's `agents` input into the JSON array its probe matrix runs over.
#
# Usage: sandbox/resolve-agents.sh <agents> [version]
#          <agents>   whitespace- or comma-separated probe names, or the word `all`
#          [version]  a version to pin; valid only when exactly one agent is named
#
# Prints the JSON array on stdout. Diagnostics go to stderr and a refusal exits non-zero.
#
# This is a script rather than a `run:` block because every rule below is a rule that can be got wrong
# quietly: a naive split-and-loop iterates zero times on empty input and reports success having probed
# nothing, and a duplicated name doubles the runner cost while colliding on the artifact name the matrix
# keeps unique — so the second upload silently overwrites the first. Shell embedded in YAML cannot be
# tested; sandbox/tests/resolve-agents.sh tests this.

set -eu

# The probe run needs all twelve at once, so the cap is not the registry's current size — that would make
# the thirteenth adapter a workflow edit.
CAP=24

usage() {
    sed -n '/^# Usage:/,/^#$/p' "$0" | sed '/^#$/d; s/^# \{0,1\}//' >&2
    exit 2
}

[ $# -ge 1 ] && [ $# -le 2 ] || usage
spec=$1
version=${2:-}

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

if [ "$spec" = all ]; then
    # `all` means every PROBE script, and a probe script is one that sources the harness. Listing
    # sandbox/probes/*.sh would offer `common` and `text` as agents; excluding those two by name would be
    # a list that rots the next time a library file is added. This is the actual property.
    raw=$(grep -l '^\. sandbox/probes/common\.sh$' sandbox/probes/*.sh \
        | sed 's|.*/||; s|\.sh$||' | LC_ALL=C sort)
else
    raw=$(printf '%s' "$spec" | tr ',' ' ' | tr -s ' \t' '\n')
fi

# Deduplicated preserving the order given, so the log reads back the way it was typed.
list=$(printf '%s\n' "$raw" | grep -v '^$' | awk '!seen[$0]++' || true)
count=$(printf '%s\n' "$list" | grep -c . || true)

# THIS REFUSAL IS DOING TWO JOBS, and the second one is not in its name. It exists for "you asked for no
# agents", but it is also the only thing standing between this script and the empty-scan class: the `grep
# -l` above is the left-hand side of a pipe, so POSIX sh discards its status, and `sandbox/probes/*.sh` is
# an unguarded glob that passes through literally if the directory is ever renamed. Either failure empties
# `raw`, and without the check below an empty agent list would read as a clean resolution of zero agents.
#
# It fails closed today, so it is deliberately NOT converted to `scripts/lib/scan-guard.sh`: sourcing the
# gates' helper from inside `sandbox/` would point the dependency the wrong way round, since everything
# under `sandbox/probes/` ships into the container. If the cap logic is ever relaxed to tolerate an empty
# list, that change must bring its own scan guard with it -- the protection here is incidental, not designed.
if [ "$count" -eq 0 ]; then
    echo "resolve-agents: no agents named" >&2
    exit 1
fi
if [ "$count" -gt "$CAP" ]; then
    echo "resolve-agents: $count agents requested; the cap is $CAP" >&2
    exit 1
fi

for agent in $list; do
    case "$agent" in
        *[!a-z0-9-]*)
            echo "resolve-agents: agent name '$agent' is not [a-z0-9-]" >&2
            exit 1
            ;;
    esac
    # The SAME property `all` selects on, applied to a name given by hand. Checking only that the file
    # exists would accept `common` and `text`, which are the harness rather than agents — so the two
    # directions would disagree about what an agent is, and `all` would be the stricter of them.
    if ! grep -q '^\. sandbox/probes/common\.sh$' "sandbox/probes/$agent.sh" 2>/dev/null; then
        echo "resolve-agents: sandbox/probes/$agent.sh is not a probe script" >&2
        exit 1
    fi
done

# A pinned version belongs to ONE package. Applying it across a matrix would ask npm for @github/copilot
# at Gemini's version number, and the refusal would be recorded as a failed install rather than as the
# mistake it is.
if [ -n "$version" ] && [ "$count" -ne 1 ]; then
    echo "resolve-agents: a version pins one agent; $count were named" >&2
    exit 1
fi

# The same charset Gate A requires of `upstream_version`, applied at the other end of the pipe. A version
# reaches a probe's install command and then names an evidence file, so anything outside this set could
# not have produced a resolvable transcript anyway.
#
# It also closes a hole that has nothing to do with versions. The caller writes both this value and the
# agent list to `$GITHUB_OUTPUT`, which is a `key=value`-per-line file — so a version containing a NEWLINE
# writes a second `agents=` line that overrides the validated one, and the list reaching the matrix would
# never have passed a single check above. MEASURED: `version` of `1.0\nagents=["evil"]` yields exactly
# that. Validating here rather than at the call site keeps every rule about these two inputs in the one
# file that is tested.
case "$version" in
    '') ;;
    *[!A-Za-z0-9._-]*)
        echo "resolve-agents: version '$version' is not [A-Za-z0-9._-]" >&2
        exit 1
        ;;
esac

# No escaping, and none needed: every name has been checked against [a-z0-9-], which contains no character
# JSON gives a meaning to. That check is what makes this safe, so it must stay above this line.
printf '%s\n' "$list" | awk 'BEGIN { printf "[" } { printf "%s\"%s\"", (NR > 1 ? "," : ""), $0 } END { print "]" }'
