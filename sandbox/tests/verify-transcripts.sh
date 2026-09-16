#!/bin/sh
# Tests for sandbox/verify-transcripts.sh, the control that makes a committed transcript evidence.
#
# Every check below corresponds to a forgery §5.3 names. They matter more than most tests in this
# repository because the thing being defended is not a behaviour but a CLAIM: a hand-written transcript
# with a real run URL, a plausible --help excerpt and a column of zero exit codes passes every SHAPE gate
# in §5, and reads in a diff as a textbook evidence refresh.

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

# A repository with a transcript, a manifest, and a faked API whose answers the test controls.
fixture() {
    work=$(mktemp -d)
    gh_dir="$work/gh"
    mkdir -p "$work/repo/docs/evidence" "$gh_dir/artifact"

    cat > "$work/gh/gh" <<'FAKE'
#!/bin/sh
# Stands in for the GitHub CLI. It answers from files the test wrote, so every rule can be exercised
# without the network and an expired artifact is just a missing file.
set -eu
case "$1 $2" in
    "api repos"*) ;;
esac
case "$*" in
    *"/jobs"*) cat "$FAKE_GH_DIR/jobs.json" ;;
    "api repos/"*"/actions/runs/"*) cat "$FAKE_GH_DIR/run.json" ;;
    "run download"*)
        [ -f "$FAKE_GH_DIR/artifact.md" ] || exit 1
        dir=$(printf '%s\n' "$@" | sed -n '/^--dir$/{n;p;}')
        cp "$FAKE_GH_DIR/artifact.md" "$dir/transcript.md"
        ;;
    *) exit 1 ;;
esac
FAKE
    chmod +x "$work/gh/gh"

    (
        cd "$work/repo"
        git init -q .
        git config user.email t@example.invalid
        git config user.name test
        # The fixture only needs commits to hang an ancestry check on. Without this, a maintainer running
        # the suite on Windows gets a CRLF warning per file per fixture, which buries the check output.
        git config core.autocrlf false
        echo harness > harness.txt
        git add -A
        git commit -qm harness
        harness_sha=$(git rev-parse HEAD)
        echo more > more.txt
        git add -A
        git commit -qm head
        git rev-parse HEAD > "$work/head"
        printf '%s' "$harness_sha" > "$work/harness"
    )
    harness_sha=$(cat "$work/harness")
    head_sha=$(cat "$work/head")

    printf 'example 1.2.3\n' > "$work/manifest"

    cat > "$work/repo/docs/evidence/example-1.2.3.md" <<TRANSCRIPT
custody: ci
run-id: 4242
run-url: https://github.com/ckir/aiprofiles/actions/runs/4242
harness-commit: $harness_sha
---
install:
  npm install --global @example/agent@1.2.3
TRANSCRIPT
    cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"

    cat > "$gh_dir/run.json" <<RUN
{"repository":{"full_name":"ckir/aiprofiles"},"path":".github/workflows/sandbox.yml","head_sha":"$harness_sha"}
RUN
    printf '{"jobs":[{"name":"Probe example","conclusion":"success"}]}\n' > "$gh_dir/jobs.json"
}

verify() {
    set +e
    out=$(cd "$work/repo" && env \
        GITHUB_REPOSITORY=ckir/aiprofiles \
        FAKE_GH_DIR="$gh_dir" \
        GH="$work/gh/gh" \
        sh "$root/sandbox/verify-transcripts.sh" "$work/manifest" "$head_sha" \
        "${1:-docs/evidence/example-1.2.3.md}" 2>&1)
    status=$?
    set -e
}

# --- the happy path -------------------------------------------------------------------------------

fixture
verify
check "a transcript matching its artifact verifies" "$status" "0"
check "and says so" "$(printf '%s' "$out" | grep -c 'ok docs/evidence/example-1.2.3.md')" "1"

# --- the forgeries §5.3 names ---------------------------------------------------------------------

# The plainest one: somebody edited the committed file. The machine assembles, the human reviews, the job
# compares; an excerpt that needs changing means the PROBE needs changing.
fixture
printf 'edited by hand\n' >> "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "an edited transcript is refused" "$status" "1"
check "and the reason is the bytes" "$(printf '%s' "$out" | grep -c 'not byte-identical')" "1"

# Renaming <id>-2.1.0.md to <id>-3.0.0.md and editing one version string reads as an evidence refresh.
# The manifest is what stops it: the version comes from the REGISTRY, so the forged name resolves to
# nothing the registry claims.
fixture
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-3.0.0.md"
verify docs/evidence/example-3.0.0.md
check "a transcript no registry row resolves to is refused" "$status" "1"
check "and the reason names the registry" \
    "$(printf '%s' "$out" | grep -c 'no adapter in the registry')" "1"

# The throwaway-branch forgery: a genuine green run, a genuine artifact, matching bytes -- produced by an
# edited probe script on a branch that never appears in the pull request.
fixture
orphan=$(cd "$work/repo" && git commit-tree -m orphan "$(git rev-parse 'HEAD^{tree}')" < /dev/null)
sed -i "s/^harness-commit: .*/harness-commit: $orphan/" "$work/repo/docs/evidence/example-1.2.3.md"
sed -i "s/\"head_sha\":\"[0-9a-f]*\"/\"head_sha\":\"$orphan\"/" "$gh_dir/run.json"
cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"
verify
check "a harness commit outside the pull request's history is refused" "$status" "1"
check "and the reason is ancestry" "$(printf '%s' "$out" | grep -c 'not an ancestor')" "1"

# --- the API assertions ---------------------------------------------------------------------------

fixture
sed -i 's|"full_name":"ckir/aiprofiles"|"full_name":"someone/else"|' "$gh_dir/run.json"
verify
check "a run from another repository is refused" "$status" "1"

fixture
sed -i 's|.github/workflows/sandbox.yml|.github/workflows/ci.yml|' "$gh_dir/run.json"
verify
check "a run of a different workflow is refused" "$status" "1"

fixture
sed -i 's/"head_sha":"[0-9a-f]*"/"head_sha":"0000000000000000000000000000000000000000"/' "$gh_dir/run.json"
verify
check "a run whose head differs from the recorded harness commit is refused" "$status" "1"

fixture
sed -i 's/^run-id: 4242/run-id: 42x/' "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "a run id that is not a run id is refused" "$status" "1"

# Twelve matrix jobs share one run id, so the RUN's conclusion says nothing about this agent.
fixture
printf '{"jobs":[{"name":"Probe other","conclusion":"success"},{"name":"Probe example","conclusion":"failure"}]}\n' \
    > "$gh_dir/jobs.json"
verify
check "a matrix job that failed is refused for a successful probe" "$status" "1"
check "and the reason names the job" "$(printf '%s' "$out" | grep -c "job 'Probe example'")" "1"

# §9 outcome 2 is a DESIGNED outcome: D13 makes a failed probe exit non-zero, so requiring success would
# reject the one transcript §9 requires to exist and Gate A requires to be present.
fixture
printf 'example unknown\n' > "$work/manifest"
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-unknown.md"
cp "$work/repo/docs/evidence/example-unknown.md" "$gh_dir/artifact.md"
printf '{"jobs":[{"name":"Probe example","conclusion":"failure"}]}\n' > "$gh_dir/jobs.json"
verify docs/evidence/example-unknown.md
check "an outcome-2 transcript verifies against a job that failed" "$status" "0"

fixture
printf 'example unknown\n' > "$work/manifest"
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-unknown.md"
cp "$work/repo/docs/evidence/example-unknown.md" "$gh_dir/artifact.md"
verify docs/evidence/example-unknown.md
check "an outcome-2 transcript from a job that SUCCEEDED is refused" "$status" "1"

# --- expiry fails closed --------------------------------------------------------------------------

# Degrading to "the run exists and was green" would check nothing that is a function of the agent, the
# version or the bytes, so the original run of <id>-1.2.3.md would satisfy it for a forged <id>-3.0.0.md.
fixture
rm "$gh_dir/artifact.md"
verify
check "an expired artifact refuses a changed transcript" "$status" "1"
check "and the reason says so" "$(printf '%s' "$out" | grep -c 'expired')" "1"

# --- the exemptions -------------------------------------------------------------------------------

fixture
sed -i 's/^custody: ci/custody: off-ci/' "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "an off-ci transcript is exempt and reviewed by hand" "$status" "0"

fixture
verify docs/superpowers/specs/whatever.md
check "a pull request touching no transcript verifies trivially" "$status" "0"

fixture
rm "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "a removed transcript is allowed by the retention rule" "$status" "0"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all verify-transcripts checks passed"
