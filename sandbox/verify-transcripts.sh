#!/bin/sh
# Binds every evidence transcript changed by a pull request to the CI run that produced it (SP4 §5.3).
#
# Usage: sandbox/verify-transcripts.sh <manifest> <head-sha> [changed-file...]
#          <manifest>       one `<id> <upstream_version>` line per real adapter, from the REGISTRY —
#                           `cargo run --example evidence-manifest`
#          <head-sha>       the pull request's head commit
#          [changed-file]   paths the pull request changes; only docs/evidence/** are considered
#
# Every gate in §5 checks a file's SHAPE. None checks that it came from anywhere, and a convincing
# transcript — real run URL copied from the public Actions tab, plausible --help excerpt, a column of zero
# exit codes — can be written by hand in minutes and passes all of them. This is the job that makes a
# transcript evidence rather than prose.
#
# `GH` is injectable so the rules below can be tested without the network; sandbox/tests/verify-transcripts.sh
# substitutes a fake. The rules are the point, and shell embedded in YAML cannot be tested.

set -eu

GH=${GH:-gh}
REPO=${GITHUB_REPOSITORY:-}

usage() {
    sed -n '3,7p' "$0" | sed 's/^# \{0,1\}//' >&2
    exit 2
}

[ $# -ge 2 ] || usage
manifest=$1
head_sha=$2
shift 2

[ -f "$manifest" ] || { echo "verify: no manifest at $manifest" >&2; exit 2; }
[ -n "$REPO" ] || { echo "verify: GITHUB_REPOSITORY is not set" >&2; exit 2; }

failures=0
fail() {
    echo "verify: FAIL $*" >&2
    failures=$((failures + 1))
}

# The changed evidence files, so a transcript nobody touched is never re-verified. That is deliberate and
# §5.3 says why: artifacts expire, and a check that cannot fail is not a check. An existing verified
# transcript stays verified; introducing or modifying one after its artifact has expired is refused.
changed=$(mktemp)
for path in "$@"; do
    case "$path" in
        docs/evidence/*.md) printf '%s\n' "$path" >> "$changed" ;;
    esac
done

if [ ! -s "$changed" ]; then
    echo "verify: no evidence transcript changed"
    rm -f "$changed"
    exit 0
fi

# field <label> <file>: the value of a `label: value` line, from the header only.
field() {
    sed -n "s/^$1: //p" "$2" | head -n1
}

verified=$(mktemp)

while read -r id version; do
    [ -n "$id" ] || continue
    file="docs/evidence/$id-$version.md"
    grep -qxF "$file" "$changed" || continue
    printf '%s\n' "$file" >> "$verified"

    # A deletion is legitimate: §7.4's retention rule replaces a superseded transcript and deletes an
    # `unknown` one once the agent installs. There is nothing to compare.
    if [ ! -f "$file" ]; then
        echo "verify: $file was removed"
        continue
    fi

    custody=$(field custody "$file")
    if [ "$custody" = off-ci ]; then
        # §7.4.1: an authenticated measurement cannot run in CI, so it carries a weaker custody block and
        # is reviewed by hand. Gate A is what stops this becoming a one-word opt-out — it requires
        # `custody: ci` for any transcript backing a ConfigIsolation or StateIsolation claim.
        echo "verify: $file is off-ci; reviewed by hand"
        continue
    fi
    if [ "$custody" != ci ]; then
        fail "$file: custody is '$custody', expected ci or off-ci"
        continue
    fi

    run_id=$(field run-id "$file")
    case "$run_id" in
        '' | *[!0-9]*)
            fail "$file: run-id '$run_id' is not a run id"
            continue
            ;;
    esac
    harness=$(field harness-commit "$file")
    case "$harness" in
        *[!0-9a-f]* | '')
            fail "$file: harness-commit '$harness' is not a commit"
            continue
            ;;
    esac
    [ "${#harness}" -eq 40 ] || { fail "$file: harness-commit is not 40 hex"; continue; }

    # Without this, a person with push access dispatches Sandbox on a throwaway branch carrying an edited
    # probe script that prints a fabricated --help. The run is genuinely green, the artifact is genuine,
    # the bytes match exactly, and the forging branch never appears in the pull request.
    #
    # Ancestry of the PR HEAD, not of the base branch. The probe is dispatched on the feature branch, so
    # its head SHA is a commit on that branch, and a commit on an open pull request's head is never an
    # ancestor of the base — the base-branch form is unsatisfiable by construction. The PR-head form says
    # what is actually meant: the harness that produced this evidence is in the history you are reviewing.
    if ! git merge-base --is-ancestor "$harness" "$head_sha" 2>/dev/null; then
        fail "$file: harness-commit $harness is not an ancestor of the pull request head"
        continue
    fi

    # §5.3 requires the run id to reach the API without passing through `${{ }}` interpolation, because it
    # is read out of a pull-request-supplied file and expression interpolation would splice it into the
    # shell the runner generates before this script ever ran. That rule binds the WORKFLOW, and the
    # workflow satisfies it by interpolating nothing: this script opens the file itself. Here the id is an
    # ordinary shell variable, already constrained to [0-9]+ above, passed as one argument.
    if ! run=$($GH api "repos/$REPO/actions/runs/$run_id" 2>/dev/null); then
        fail "$file: run $run_id could not be read from $REPO"
        continue
    fi

    got_repo=$(printf '%s' "$run" | jq -r '.repository.full_name // ""')
    got_path=$(printf '%s' "$run" | jq -r '.path // ""')
    got_sha=$(printf '%s' "$run" | jq -r '.head_sha // ""')

    [ "$got_repo" = "$REPO" ] || fail "$file: run $run_id belongs to $got_repo, not $REPO"
    [ "$got_path" = ".github/workflows/sandbox.yml" ] \
        || fail "$file: run $run_id is $got_path, not the Sandbox workflow"
    [ "$got_sha" = "$harness" ] \
        || fail "$file: run $run_id ran at $got_sha, but the transcript records $harness"

    # The MATRIX JOB's conclusion, not the run's. Twelve matrix jobs share one run id, and §9 outcome 2 is
    # a DESIGNED outcome in which a probe legitimately fails — so a run-level check would let one
    # uninstallable agent invalidate eleven good transcripts.
    if ! jobs=$($GH api "repos/$REPO/actions/runs/$run_id/jobs" --paginate 2>/dev/null); then
        fail "$file: the jobs of run $run_id could not be read"
        continue
    fi
    conclusion=$(printf '%s' "$jobs" | jq -r --arg name "Probe $id" \
        '[.jobs[]? | select(.name == $name)] | first | .conclusion // ""')

    # An outcome-2 transcript is verified against a job that CONCLUDED FAILURE, which is the whole point
    # of outcome 2: D13 makes a failed probe exit non-zero, so requiring success here would reject the one
    # transcript §9 requires to exist and Gate A requires to be present.
    if [ "$version" = unknown ]; then
        expected=failure
    else
        expected=success
    fi
    if [ "$conclusion" != "$expected" ]; then
        fail "$file: job 'Probe $id' concluded '$conclusion', expected '$expected'"
        continue
    fi

    # Expiry FAILS CLOSED. Degrading to "the run exists and was green" would check nothing that is a
    # function of the agent, the version, or the bytes — so the original run of <id>-2.1.0.md would
    # satisfy it for a forged <id>-3.0.0.md, restoring both forgeries this whole section exists to stop.
    dir=$(mktemp -d)
    if ! $GH run download "$run_id" --name "sandbox-transcript-$id" --dir "$dir" >/dev/null 2>&1; then
        fail "$file: artifact sandbox-transcript-$id is unavailable (expired?); a transcript cannot be introduced or modified after its artifact has gone"
        rm -rf "$dir"
        continue
    fi

    downloaded=$(find "$dir" -type f -name '*.md' | head -n1)
    if [ -z "$downloaded" ]; then
        fail "$file: artifact sandbox-transcript-$id holds no transcript"
        rm -rf "$dir"
        continue
    fi
    if cmp -s "$file" "$downloaded"; then
        echo "verify: ok $file"
    else
        # The machine assembles, the human reviews, the job compares. A mismatch means somebody edited a
        # transcript; an excerpt that needs changing means the PROBE needs changing and the probe re-runs.
        fail "$file: not byte-identical to the artifact from run $run_id"
    fi
    rm -rf "$dir"
done < "$manifest"

# A changed evidence file that no registry row resolves to. Without this, adding a transcript for an agent
# the registry does not carry is simply never checked by anything here.
while read -r path; do
    grep -qxF "$path" "$verified" && continue
    case "$path" in
        docs/evidence/README.md) continue ;;
    esac
    if [ -f "$path" ] && [ "$(field custody "$path")" = off-ci ]; then
        echo "verify: $path is off-ci; reviewed by hand"
        continue
    fi
    fail "$path: no adapter in the registry resolves to this transcript"
done < "$changed"

rm -f "$changed" "$verified"

if [ "$failures" -ne 0 ]; then
    echo "verify: $failures transcript check(s) failed" >&2
    exit 1
fi
echo "verify: all changed transcripts are bound to their runs"
