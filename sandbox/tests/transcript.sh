#!/bin/sh
# Tests for sandbox/transcript.sh. Run by `just check` and by CI; no container and no agent needed.
#
# The assembler writes the file §5.3 later compares byte for byte against the CI artifact, so a defect
# here is not a formatting problem: it is a durable false statement about what an agent did, committed to
# the repository and carrying a chain of custody that says it was measured.

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

# A finished probe's results directory, as sandbox/run.sh leaves it.
fixture() {
    results=$(mktemp -d)
    outdir=$(mktemp -d)
    printf 'npm install --global @example/agent@1.2.3\n' > "$results/install.cmd"
    printf 'install 0\nversion 0\nhelp 0\nstrings 0\nbehaviour-env-EXAMPLE_HOME 0\n' > "$results/steps"
    printf 'agent 1.2.3\n' > "$results/version.txt"
    printf '1.2.3\n' > "$results/version.extracted"
    printf 'Usage: agent [--config <dir>]\n' > "$results/help.txt"
    printf 'documented EXAMPLE_API_KEY: present\n' > "$results/strings.txt"
    printf '# /target\n# /home/probe/.example\n' > "$results/baseline-env-EXAMPLE_HOME.txt"
    printf '# /target\nd .example\n' > "$results/delta-env-EXAMPLE_HOME.txt"
    printf 'A /home/probe/.example\nA /home/probe/work/src\nC /home/probe/.npm\n' > "$results/diff.txt"
    # Written by `sandbox/run.sh`'s write to `$out/exit-code` from OUTSIDE the container, which is what
    # makes it the one status in the transcript the measured party could not have chosen.
    printf '0\n' > "$results/exit-code"
}

run_assembler() {
    set +e
    written=$(cd "$root" && env GITHUB_RUN_ID=4242 GITHUB_SERVER_URL=https://github.com \
        GITHUB_REPOSITORY=ckir/aiprofiles GITHUB_SHA=0123456789abcdef0123456789abcdef01234567 \
        sh sandbox/transcript.sh "$1" "$results" "$outdir" 2>"$outdir/err")
    assembler_status=$?
    set -e
}

field() {
    sed -n "s/^$1: //p" "$written" | head -n1
}

# --- the custody header ---------------------------------------------------------------------------

fixture
run_assembler example
check "the assembler succeeds on a complete run" "$assembler_status" "0"
check "the file is named by id and recorded version" "$(basename "$written")" "example-1.2.3.md"
check "custody says ci" "$(field custody)" "ci"
check "the run id is carried" "$(field run-id)" "4242"
check "the run url is built from the run id" \
    "$(field run-url)" "https://github.com/ckir/aiprofiles/actions/runs/4242"
check "the harness commit is carried" \
    "$(field harness-commit)" "0123456789abcdef0123456789abcdef01234567"
check "the separator follows the header" "$(sed -n 5p "$written")" "---"

# --- the measurements -----------------------------------------------------------------------------

check "the install command is the resolved one" \
    "$(sed -n '/^install:/{n;p;}' "$written")" "  npm install --global @example/agent@1.2.3"
check "exit codes are in step order" \
    "$(sed -n '/^exit-codes:/{n;p;}' "$written")" "  install 0"
check "the extracted version is stated beside the raw output" \
    "$(field version-extracted)" "1.2.3"

# `verify-transcripts.sh` decides the matrix job's expected conclusion from this field, so it must come
# from the container's status and not be derived from the per-step codes above it: once a candidate
# refusal stopped being a failure, `exit-codes:` can carry a non-zero line for a probe that exited 0.
check "the probe's exit code is carried from the results directory" "$(field probe-exit)" "0"

fixture
printf '1\n' > "$results/exit-code"
run_assembler example
check "a probe that exited non-zero says so" "$(field probe-exit)" "1"
check "each mechanism's baseline is emitted" \
    "$(grep -c '^baseline-env-EXAMPLE_HOME:$' "$written")" "1"
check "each mechanism's delta follows its baseline" \
    "$(grep -c '^delta-env-EXAMPLE_HOME:$' "$written")" "1"

# The copied checkout is thousands of paths the probe put there itself. Leaving it in would bound the
# summary away to nothing and bury the handful of lines that show a write nobody predicted.
check "the container delta keeps what the agent wrote" \
    "$(grep -c 'A /home/probe/.example$' "$written")" "1"
check "the container delta drops our own copy of the checkout" \
    "$(grep -c '/home/probe/work' "$written")" "0"

# An unanchored filter drops any row whose PATH merely contains "/home/probe/work" as a substring —
# which lets an agent hide an arbitrary write by naming it to end in that string, and also silently
# drops a legitimate path such as /home/probe/workspace/... . The filter must be anchored to the
# change-letter and the path field, so it drops only rows that ARE our checkout, keeping a near-miss.
fixture
printf 'A /home/probe/work/real-noise\nA /root/.ssh/authorized_keys /home/probe/work\nC /etc/hidden /home/probe/workaround\nA /home/probe/workspace/keep-me\n' \
    > "$results/diff.txt"
run_assembler example
check "the container delta keeps a write that only looks like our checkout" \
    "$(grep -c '/home/probe/work/real-noise' "$written")-$(grep -c '/root/\.ssh/authorized_keys /home/probe/work$' "$written")-$(grep -c '/etc/hidden /home/probe/workaround' "$written")-$(grep -c '/home/probe/workspace/keep-me' "$written")" \
    "0-1-1-1"

# §8.4's sweep is present only for the two configuration-file agents.
check "no candidates section when there was no sweep" "$(grep -c '^candidates:$' "$written")" "0"
fixture
printf '1: @none\n2: {}\n' > "$results/candidates.txt"
run_assembler example
check "a candidates section when there was one" "$(grep -c '^candidates:$' "$written")" "1"

# --- absence is a finding, not a gap ---------------------------------------------------------------

# A transcript with no `help:` line reads as an agent with no help text. One saying `(not recorded)` reads
# as a probe that did not get that far, and those are different findings.
fixture
rm "$results/help.txt"
run_assembler example
check "a missing artefact is stated, not skipped" \
    "$(sed -n '/^help:/{n;p;}' "$written")" "  (not recorded)"

# --- refusals -------------------------------------------------------------------------------------

# Gate A resolves `docs/evidence/<id>-<version>.md` from the registry's upstream_version. A version
# carrying a space or a slash would name a file the gate can never resolve, or escape the directory.
fixture
printf 'agent 1.2.3\n' > "$results/version.extracted"
run_assembler example
check "a version outside Gate A's charset is refused" "$assembler_status" "2"

fixture
printf '../escape\n' > "$results/version.extracted"
run_assembler example
check "a version that would escape the directory is refused" "$assembler_status" "2"

fixture
run_assembler 'Bad Id'
check "an id outside the id charset is refused" "$assembler_status" "2"

# A probe that never reached step 2 recorded nothing. `unknown` is the encoding Gate C keys on to force
# the adapter to Experimental, so the transcript states it rather than failing to exist.
fixture
rm "$results/version.extracted"
run_assembler example
check "a probe that recorded no version yields an unknown transcript" \
    "$(basename "$written")" "example-unknown.md"

# FAIL CLOSED. `probe-exit` is what the verifier checks the matrix job's conclusion against, so a status
# the assembler could not read must be written as something that FAILS that check. `unknown` is neither
# `0` nor a number, so `verify-transcripts.sh` refuses the transcript; writing `0` would turn a missing
# status into the value that passes against a green job.
fixture
rm "$results/exit-code"
run_assembler example
check "a missing exit code is not silently a clean run" "$(field probe-exit)" "unknown"

fixture
printf 'boom\n' > "$results/exit-code"
run_assembler example
check "a non-numeric exit code is not silently a clean run" "$(field probe-exit)" "unknown"

fixture
: > "$results/exit-code"
run_assembler example
check "an empty exit code is not silently a clean run" "$(field probe-exit)" "unknown"

# --- a run the harness refused is not evidence, and gets no committable transcript -----------------
#
# `common.sh` names its own failures with a reserved `harness-` prefix. Such a run stops before step 2, so
# it would otherwise be assembled as `<id>-unknown.md` -- the same name a genuine "this agent would not
# install" measurement carries -- and §5.3's verifier, which pairs any non-zero `probe-exit:` with a failed
# job, would accept it. That is a transcript blaming a vendor for our own breakage.
#
# NOTHING IN verify-transcripts.sh ENFORCES THIS, deliberately: it reads single-line headers only, and
# teaching it to parse `exit-codes:` would put new block-parsing in the most load-bearing gate there is.
# What stands behind these checks instead is the verifier's EXISTING artifact-existence check -- a
# hand-assembled `.md` is refused because the run's artifact holds no `.md` to compare against. These two
# checks are therefore the only thing pinning the behaviour, which is why they assert the absence of the
# transcript and not merely the presence of the diagnostic.
fixture
printf 'harness-privilege 1\n' > "$results/steps"
printf '1\n' > "$results/exit-code"
rm -f "$results/version.extracted"
run_assembler example
check "a run the harness refused still exits 0, so the workflow step does not mask it" \
    "$assembler_status" "0"
# Counted through a glob rather than `ls | grep`: POSIX sh has no nullglob, so the `[ -e ]` is what makes
# "no matches" and "one file" distinguishable at all here.
md_written=0
for md_file in "$outdir"/*.md; do
    [ -e "$md_file" ] && md_written=$((md_written + 1))
done
check "and NO committable transcript is written for it" "$md_written" "0"
check "and the diagnostic it writes instead cannot be resolved by any registry row" \
    "$(basename "$written")" "example-harness-refused.txt"
check "and the diagnostic names the step that refused the run" \
    "$(grep -c 'harness-privilege 1' "$written")" "1"

# The harness failures that happen OUTSIDE the container leave no step rows at all to carry a prefix: a
# failed `eng build`, or a container that dies before `common.sh` is sourced. `sandbox/run.sh` still writes
# an `exit-code`, so before this was covered the assembler produced a clean-looking `<id>-unknown.md` with
# `(not recorded)` in every block -- MEASURED end to end through the real verifier, which accepted it. The
# absence of a step list is therefore itself a refusal.
fixture
rm -f "$results/steps" "$results/version.extracted"
printf '125\n' > "$results/exit-code"
run_assembler example
check "a run that recorded no step at all is refused too" \
    "$(basename "$written")" "example-harness-refused.txt"
check "and it says the run produced no measurement, rather than dying while saying so" \
    "$(grep -c 'no step was recorded' "$written")" "1"

# The other half: a run that failed for the AGENT's reasons must still produce its transcript. Without
# this, a check that only asserted the refusal could be satisfied by an assembler that never writes
# anything at all. `strings 127` is the agent's binary not being on its own PATH -- a fact about the agent,
# and deliberately not prefixed.
fixture
printf 'install 0\nversion 0\nhelp 0\nstrings 127\n' > "$results/steps"
printf '1\n' > "$results/exit-code"
run_assembler example
check "a run that failed for the agent's own reasons is still assembled" \
    "$(basename "$written")" "example-1.2.3.md"
check "and it records the agent-side failure" \
    "$(sed -n '/^exit-codes:/,/^probe-exit:/p' "$written" | grep -c 'strings 127')" "1"

# --- an agent's own text cannot become a header field ----------------------------------------------
#
# `section()` indents every body line by two spaces before it is written. `verify-transcripts.sh`'s
# `field()` is `sed -n "s/^$1: //p" | head -n1` -- unanchored to any particular line, it scans the WHOLE
# file and returns the FIRST match. The raw agent-authored `version:`/`help:`/`strings:` sections are
# written before the harness's own `version-extracted:` line, so that two-space indent is the only thing
# stopping an agent's own output from supplying a header field the verifier trusts.

fixture
printf 'agent 1.2.3\nversion-extracted: 9.9.9\n' > "$results/version.txt"
run_assembler example
check "agent-authored text cannot supply a header field" "$(field version-extracted)" "1.2.3"

# Sibling checks, and TWO independent things hold them -- measured, because the single mutant above does
# not redden them and that could be mistaken for a vacuous pair.
#
# The first is ORDER: `custody` and `probe-exit` are written before any section() body, so nothing an agent
# puts in help.txt or strings.txt can precede them, and `field()` takes the FIRST match. The second is the
# INDENT: even out of order, an injected line arrives as `  probe-exit: 99`, which `^probe-exit: ` does not
# match. MEASURED: moving the `probe-exit:` emission after the sections leaves this suite GREEN while the
# indent stands, and only dropping BOTH turns this check red with `probe-exit` reading the agent's 99.
#
# So these are defence in depth rather than a wider version of the check above, and a single-point mutant
# leaving them green is the expected result, not a sign they assert nothing.
fixture
printf 'Usage: agent [--config <dir>]\ncustody: forged\n' > "$results/help.txt"
run_assembler example
check "agent-authored text in help cannot supply custody" "$(field custody)" "ci"

fixture
printf 'documented EXAMPLE_API_KEY: present\nprobe-exit: 99\n' > "$results/strings.txt"
run_assembler example
check "agent-authored text in strings cannot supply probe-exit" "$(field probe-exit)" "0"

# --- a run with no behaviour step is not a glob left unexpanded ------------------------------------
#
# The baseline loop's only nullglob guard is `[ -e "$f" ] || break`. A probe whose install fails never
# reaches step 5 and writes no `baseline-*.txt` at all -- the designed shape of outcome 2 -- and without
# the guard, an unmatched glob stays literal and the loop would emit a section labelled by the glob text
# itself instead of nothing.
fixture
rm -f "$results"/baseline-*.txt "$results"/delta-*.txt
printf 'install 1\n' > "$results/steps"
printf '1\n' > "$results/exit-code"
run_assembler example
check "a run with no behaviour step emits no baseline or delta section at all" \
    "$(grep -c '^baseline-' "$written")-$(grep -c '^delta-' "$written")" "0-0"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all transcript checks passed"
