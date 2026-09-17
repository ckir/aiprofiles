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
        # The NAME matters as much as the bytes: `upload-artifact` with `path: transcript/` roots
        # `<id>-<version>.md` at the artifact root, so the test must be able to put a wrong name there.
        cp "$FAKE_GH_DIR/artifact.md" "$dir/$FAKE_ARTIFACT_NAME"
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

    # The name the probe itself chose. `sandbox/transcript.sh`'s `out=` assignment writes `<id>-<version>.md`,
    # so this is what the artifact holds unless a test deliberately puts something else there.
    artifact_name=example-1.2.3.md

    cat > "$work/repo/docs/evidence/example-1.2.3.md" <<TRANSCRIPT
custody: ci
run-id: 4242
run-url: https://github.com/ckir/aiprofiles/actions/runs/4242
harness-commit: $harness_sha
---
install:
  npm install --global @example/agent@1.2.3
probe-exit: 0
version-extracted: 1.2.3
TRANSCRIPT
    cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"

    cat > "$gh_dir/run.json" <<RUN
{"repository":{"full_name":"ckir/aiprofiles"},"path":".github/workflows/sandbox.yml","head_sha":"$harness_sha"}
RUN
    printf '{"jobs":[{"name":"Probe example","conclusion":"success"}]}\n' > "$gh_dir/jobs.json"
}

# verify [changed-path...]: run the script over the fixture. A pull request usually changes more than one
# evidence file — a refresh deletes one and adds another — so every argument is passed through.
verify() {
    [ $# -gt 0 ] || set -- docs/evidence/example-1.2.3.md
    set +e
    out=$(cd "$work/repo" && env \
        GITHUB_REPOSITORY=ckir/aiprofiles \
        FAKE_GH_DIR="$gh_dir" \
        FAKE_ARTIFACT_NAME="$artifact_name" \
        GH="$work/gh/gh" \
        sh "$root/sandbox/verify-transcripts.sh" "$work/manifest" "$head_sha" "$@" 2>&1)
    status=$?
    set -e
}

# The §9 outcome-2 shape: the probe failed, so `sandbox/transcript.sh` recorded `unknown` as the version,
# named the file for it, and wrote `unknown` into `version-extracted` too — and, because the probe failed,
# `sandbox/run.sh` recorded a non-zero container status in `probe-exit`. The version is no longer what the
# conclusion rule reads; the exit code is.
unknown_fixture() {
    fixture
    printf 'example unknown\n' > "$work/manifest"
    sed -i 's/^version-extracted: .*/version-extracted: unknown/' \
        "$work/repo/docs/evidence/example-1.2.3.md"
    sed -i 's/^probe-exit: .*/probe-exit: 1/' "$work/repo/docs/evidence/example-1.2.3.md"
    mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-unknown.md"
    cp "$work/repo/docs/evidence/example-unknown.md" "$gh_dir/artifact.md"
    artifact_name=example-unknown.md
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

# --- the conclusion rule ----------------------------------------------------------------------------
#
# Which conclusion a transcript requires is decided by the status the PROBE EXITED WITH — `probe-exit`,
# written by `sandbox/run.sh` from outside the container and carried into the file by
# `sandbox/transcript.sh`. Keying it on the version instead ("unknown means the probe failed") left §9's
# outcomes 3 and 4 with no committable form at all, which is the defect the checks below pin.
#
# Keying it on the `exit-codes:` block was the obvious alternative and is wrong: an acceptance sweep's
# refusals appear there and are not failures, so a probe that legitimately exits 0 would be made to demand
# a failed job.

# §9 outcome 2 is a DESIGNED outcome: D13 makes a failed probe exit non-zero, so requiring success would
# reject the one transcript §9 requires to exist and Gate A requires to be present.
unknown_fixture
printf '{"jobs":[{"name":"Probe example","conclusion":"failure"}]}\n' > "$gh_dir/jobs.json"
verify docs/evidence/example-unknown.md
check "an outcome-2 transcript verifies against a job that failed" "$status" "0"

unknown_fixture
verify docs/evidence/example-unknown.md
check "an outcome-2 transcript from a job that SUCCEEDED is refused" "$status" "1"

# Outcomes 3 and 4: the agent installed, the version IS known, and a step was refused, so the probe
# exited non-zero and the job concluded failure. Under the version rule this transcript could not exist —
# a known version demanded a green job — and it is the shape aider, amp and continue produce every run.
fixture
sed -i 's/^probe-exit: .*/probe-exit: 1/' "$work/repo/docs/evidence/example-1.2.3.md"
cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"
printf '{"jobs":[{"name":"Probe example","conclusion":"failure"}]}\n' > "$gh_dir/jobs.json"
verify
check "an outcome-3 transcript with a known version verifies against a job that failed" "$status" "0"

# The status in the file has to AGREE with the run, in both directions.
fixture
sed -i 's/^probe-exit: .*/probe-exit: 1/' "$work/repo/docs/evidence/example-1.2.3.md"
cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"
verify
check "a non-zero probe-exit against a job that SUCCEEDED is refused" "$status" "1"
check "and the reason names the conclusion it expected" \
    "$(printf '%s' "$out" | grep -c "concluded 'success', expected 'failure'")" "1"

fixture
printf '{"jobs":[{"name":"Probe example","conclusion":"failure"}]}\n' > "$gh_dir/jobs.json"
verify
check "a zero probe-exit against a job that FAILED is refused" "$status" "1"
check "and the reason names the conclusion it expected there too" \
    "$(printf '%s' "$out" | grep -c "concluded 'failure', expected 'success'")" "1"

# FAIL CLOSED. `sandbox/transcript.sh` writes `unknown` when it could not read the container's status, and
# a hand-written transcript can simply leave the line out. Neither may pass: defaulting to either
# conclusion would let a file stating no status at all satisfy some job.
fixture
sed -i '/^probe-exit: /d' "$work/repo/docs/evidence/example-1.2.3.md"
cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"
verify
check "a transcript carrying no probe-exit is refused" "$status" "1"
check "and the reason names probe-exit" \
    "$(printf '%s' "$out" | grep -c "probe-exit '' is not a probe exit code")" "1"

fixture
sed -i 's/^probe-exit: .*/probe-exit: unknown/' "$work/repo/docs/evidence/example-1.2.3.md"
cp "$work/repo/docs/evidence/example-1.2.3.md" "$gh_dir/artifact.md"
verify
check "a transcript whose probe-exit is not a number is refused" "$status" "1"
check "and the reason quotes the value" \
    "$(printf '%s' "$out" | grep -c "probe-exit 'unknown' is not a probe exit code")" "1"

# --- expiry fails closed --------------------------------------------------------------------------

# Degrading to "the run exists and was green" would check nothing that is a function of the agent, the
# version or the bytes, so the original run of <id>-1.2.3.md would satisfy it for a forged <id>-3.0.0.md.
fixture
rm "$gh_dir/artifact.md"
verify
check "an expired artifact refuses a changed transcript" "$status" "1"
check "and the reason says so" "$(printf '%s' "$out" | grep -c 'expired')" "1"

# --- the version bindings -------------------------------------------------------------------------

# The relabel, MEASURED against the unbound script: take a verified example-1.2.3.md, bump the registry to
# 3.0.0, `git mv` the file. ZERO bytes change, so every check above is satisfied by the 1.2.3 run and the
# repository ends up carrying a `measured:` basis for 3.0.0 that is a measurement of 1.2.3. The expiry
# rule does not stop this; only a binding between the version and what the probe produced does.
fixture
printf 'example 3.0.0\n' > "$work/manifest"
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-3.0.0.md"
verify docs/evidence/example-3.0.0.md
check "a transcript whose version-extracted disagrees with the registry is refused" "$status" "1"
check "and the reason names version-extracted" \
    "$(printf '%s' "$out" | grep -c "version-extracted is '1.2.3', but the registry says '3.0.0'")" "1"

# The same forgery with the body edited to match. The bytes below are IDENTICAL to the artifact's; the
# only thing wrong is the artifact's own file name, which is the one thing in it a pull request cannot
# rewrite. Taking whatever .md the artifact happens to hold would pass this.
fixture
printf 'example 3.0.0\n' > "$work/manifest"
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-3.0.0.md"
sed -i 's/^version-extracted: .*/version-extracted: 3.0.0/' "$work/repo/docs/evidence/example-3.0.0.md"
cp "$work/repo/docs/evidence/example-3.0.0.md" "$gh_dir/artifact.md"
verify docs/evidence/example-3.0.0.md
check "an artifact holding a differently-named transcript is refused" "$status" "1"
check "and the reason names the file the artifact should hold" \
    "$(printf '%s' "$out" | grep -c 'holds no example-3.0.0.md')" "1"

# --- the exemptions -------------------------------------------------------------------------------

# `custody: off-ci` on a registry-resolved transcript is not an exemption, it is a one-word opt-out of the
# whole of §5.3 — run id, harness commit, ancestry, the run API, the matrix conclusion, the artifact and
# the bytes — available to anyone who can open a pull request. Nothing outside this script enforces it:
# `gate_a_shape` in crates/agent-profile/src/adapter/gate.rs carries no transcript clause.
fixture
sed -i 's/^custody: ci/custody: off-ci/' "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "an off-ci transcript the registry resolves to is refused" "$status" "1"
check "and the reason is custody" "$(printf '%s' "$out" | grep -c "custody is 'off-ci'")" "1"

# The legitimate off-ci traffic, and the only kind: §7.4.1's authenticated measurement, which cannot run
# in CI. It is named `<id>-<version>-credentials.md` and sits BESIDE the CI transcript rather than
# replacing it, so no registry row can resolve it and it lands in the orphan loop.
fixture
sed 's/^custody: ci/custody: off-ci/' "$work/repo/docs/evidence/example-1.2.3.md" \
    > "$work/repo/docs/evidence/example-1.2.3-credentials.md"
verify docs/evidence/example-1.2.3-credentials.md
check "an off-ci credentials transcript is exempt and reviewed by hand" "$status" "0"
check "and says it was reviewed by hand" \
    "$(printf '%s' "$out" | grep -c 'example-1.2.3-credentials.md is off-ci; reviewed by hand')" "1"

# Any other name asking for that exemption is asking to skip §5.3 under a different spelling.
fixture
sed 's/^custody: ci/custody: off-ci/' "$work/repo/docs/evidence/example-1.2.3.md" \
    > "$work/repo/docs/evidence/example-9.9.9.md"
verify docs/evidence/example-9.9.9.md
check "an off-ci orphan that is not a credentials transcript is refused" "$status" "1"
check "and the reason names the registry" \
    "$(printf '%s' "$out" | grep -c 'no adapter in the registry')" "1"

fixture
verify docs/superpowers/specs/whatever.md
check "a pull request touching no transcript verifies trivially" "$status" "0"

# --- the retention rule ---------------------------------------------------------------------------

# Both flows docs/evidence/README.md documents delete a file whose registry row has ALREADY moved on, so
# the deleted path resolves to nothing and reaches the ORPHAN loop. Refusing it would redden every
# evidence refresh in the repository; a deletion cannot introduce false evidence.
fixture
printf 'example 3.0.0\n' > "$work/manifest"
mv "$work/repo/docs/evidence/example-1.2.3.md" "$work/repo/docs/evidence/example-3.0.0.md"
sed -i 's/^version-extracted: .*/version-extracted: 3.0.0/' "$work/repo/docs/evidence/example-3.0.0.md"
cp "$work/repo/docs/evidence/example-3.0.0.md" "$gh_dir/artifact.md"
artifact_name=example-3.0.0.md
verify docs/evidence/example-1.2.3.md docs/evidence/example-3.0.0.md
check "a refresh that supersedes a transcript verifies" "$status" "0"
check "and says the superseded one was removed" \
    "$(printf '%s' "$out" | grep -c 'docs/evidence/example-1.2.3.md was removed')" "1"

# §9 outcome 2's exit: the agent installs at last, so the `unknown` transcript is deleted and the real one
# added. This is the expected shape of the first successful re-probe of any of the nine new agents.
fixture
verify docs/evidence/example-unknown.md docs/evidence/example-1.2.3.md
check "deleting an unknown transcript when the real one arrives verifies" "$status" "0"
check "and says the unknown one was removed" \
    "$(printf '%s' "$out" | grep -c 'docs/evidence/example-unknown.md was removed')" "1"

# The other deletion shape, which takes the `:73-76` branch instead: the registry STILL names the file the
# pull request deleted. There is nothing to compare, so §7.4's retention rule allows it.
fixture
rm "$work/repo/docs/evidence/example-1.2.3.md"
verify
check "a transcript the registry still names but the pull request deleted is allowed" "$status" "0"
check "and says it was removed" \
    "$(printf '%s' "$out" | grep -c 'docs/evidence/example-1.2.3.md was removed')" "1"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all verify-transcripts checks passed"
