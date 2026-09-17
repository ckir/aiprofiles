#!/bin/sh
# Assembles one evidence transcript from a finished probe run.
#
# Usage: sandbox/transcript.sh <agent-id> <results-dir> [output-dir]
#          <results-dir>  target/sandbox/probe-<id>-<stamp>/, what sandbox/run.sh leaves behind
#          [output-dir]   docs/evidence by default
#
# It runs as a WORKFLOW STEP, not inside the container, and that is forced rather than chosen: the
# container gets no `--env` (neither `eng run` invocation in `sandbox/run.sh` passes one), so GITHUB_RUN_ID
# is not visible to a probe and the
# custody header cannot be written where the measurements are taken.
#
# It writes `<id>-<version>.md` whole, and the maintainer commits those bytes unaltered. §5.3's
# verification job then compares the committed file against the artifact byte for byte, which is what
# makes the custody header load-bearing instead of decorative — every field above the separator is bound
# by the same comparison as the measurements below it.

set -eu

usage() {
    sed -n '/^# Usage:/,/^#$/p' "$0" | sed '/^#$/d; s/^# \{0,1\}//' >&2
    exit 2
}

[ $# -ge 2 ] && [ $# -le 3 ] || usage
id=$1
results=$2
outdir=${3:-docs/evidence}

case "$id" in
    '' | *[!a-z0-9-]*) echo "transcript: agent ids are [a-z0-9-]" >&2; exit 2 ;;
esac
[ -d "$results" ] || { echo "transcript: no such results directory: $results" >&2; exit 2; }

root=$(cd "$(dirname "$0")/.." && pwd)
. "$root/sandbox/probes/text.sh"

# The version names the file, so §5.3 can resolve one transcript from the registry's `upstream_version`.
# A probe that recorded none wrote `unknown`, which Gate C keys on; a missing file means the probe did not
# reach step 2 at all, and `unknown` is the honest encoding of that too.
version=unknown
if [ -s "$results/version.extracted" ]; then
    version=$(head -n1 "$results/version.extracted")
fi
case "$version" in
    '' | *[!A-Za-z0-9._-]*)
        echo "transcript: recorded version '$version' is outside Gate A's charset" >&2
        exit 2
        ;;
esac

# The probe's own exit status, as `sandbox/run.sh:192` recorded it from OUTSIDE the container — the one
# number in this file that the measured party could not have written. It is what `verify-transcripts.sh`
# compares the matrix job's conclusion against, and it is why §9's outcomes 3 and 4 (agent installed,
# version known, a step refused) have a committable form at all: the version no longer has to stand in for
# the run's status.
#
# FAILS CLOSED. A missing, empty or non-numeric file yields `unknown`, which is not `0` and not a number,
# so the verifier refuses the transcript rather than reading the gap as a clean run. Writing `0` there
# would turn an absent status into the value that passes.
probe_exit=unknown
if [ -s "$results/exit-code" ]; then
    probe_exit=$(head -n1 "$results/exit-code")
fi
case "$probe_exit" in
    '' | *[!0-9]*) probe_exit=unknown ;;
esac

mkdir -p "$outdir"
out="$outdir/$id-$version.md"

# section <label> <file> <max-lines>: one labelled block, stripped and bounded, or an explicit absence.
#
# A missing artefact is PRINTED rather than skipped. A transcript with no `help:` line reads as an agent
# with no help text; one saying `(not recorded)` reads as a probe that did not get that far, and those are
# different findings.
section() {
    printf '%s:\n' "$1"
    if [ -s "$2" ]; then
        probe_excerpt "$3" < "$2" | sed 's/^/  /'
    else
        echo "  (not recorded)"
    fi
}

{
    # --- custody, bound by the byte comparison exactly as the measurements are -------------------
    echo "custody: ci"
    echo "run-id: ${GITHUB_RUN_ID:-unknown}"
    echo "run-url: ${GITHUB_SERVER_URL:-https://github.com}/${GITHUB_REPOSITORY:-unknown}/actions/runs/${GITHUB_RUN_ID:-unknown}"
    echo "harness-commit: ${GITHUB_SHA:-unknown}"
    echo "---"

    # --- what was run, and what it returned -----------------------------------------------------
    printf 'install:\n'
    if [ -s "$results/install.cmd" ]; then
        sed 's/^/  /' "$results/install.cmd"
    else
        echo "  (not recorded)"
    fi

    printf 'exit-codes:\n'
    if [ -s "$results/steps" ]; then
        # In step order, which is the order the probe ran them. A directory listing would sort
        # `behaviour` before `install` and read as though the agent was launched before it existed.
        sed 's/^/  /' "$results/steps"
    else
        echo "  (not recorded)"
    fi

    # Beside the per-step codes, and deliberately not derived from them: once a candidate refusal stopped
    # being a failure, the `exit-codes:` block above can carry a non-zero line for a probe that exited 0.
    # This is the status the container returned, which is the status the matrix job concluded from.
    echo "probe-exit: $probe_exit"

    section version "$results/version.txt" 20
    echo "version-extracted: $version"
    section help "$results/help.txt" 200
    section strings "$results/strings.txt" 200

    # --- one baseline/delta pair per mechanism under test ---------------------------------------
    for f in "$results"/baseline-*.txt; do
        [ -e "$f" ] || break
        label=$(basename "$f" .txt)
        section "$label" "$f" 200
        section "delta-${label#baseline-}" "$results/delta-${label#baseline-}.txt" 200
    done

    # §8.4's acceptance sweep, present only for the configuration-file agents.
    if [ -s "$results/candidates.txt" ]; then
        section candidates "$results/candidates.txt" 40
    fi

    # --- the unscoped observation ---------------------------------------------------------------
    # The only record of a write to a location nobody predicted, which is the only way §9 outcome 4 —
    # the agent exposes a DIFFERENT mechanism from the claimed one — is ever detected. The `delta:`
    # sections above see only the locations chosen in advance.
    #
    # /home/probe/work is dropped because it is OUR copy of the checkout (`sandbox/run.sh`'s `copy=`), some
    # thousands of paths the probe put there itself. Leaving it in would bound the summary away to
    # nothing and bury the handful of lines that matter.
    printf 'container-delta:\n'
    if [ -s "$results/diff.txt" ]; then
        grep -v '^[ACD] /home/probe/work\(/\|$\)' "$results/diff.txt" \
            | probe_excerpt 300 \
            | sed 's/^/  /'
    else
        echo "  (not recorded)"
    fi
} > "$out"

echo "$out"
