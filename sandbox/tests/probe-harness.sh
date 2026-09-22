#!/bin/sh
# Tests for sandbox/probes/common.sh and sandbox/probes/text.sh. Run by `just check` and by CI; no
# container and no agent needed.
#
# The defect these exist for: `probe_record` used to end with `set -e`, and a shell function returns the
# status of its last command, so it always returned 0. Every probe script ends in a recorded command, so a
# probe whose install failed and whose every later step returned 127 still exited 0 and the job went green.
# A test of `probe_record` alone would not have caught it — the status that mattered was the SCRIPT's.

# shellcheck disable=SC2016 # a probe body is single-quoted ON PURPOSE: it must expand in the generated
# script, not in this shell. Expanding it here would substitute the test runner's empty $PROBE_OUT and
# every check would then assert against a path in the filesystem root.

set -eu
root=$(cd "$(dirname "$0")/../.." && pwd)
failures=0

# The repository's one answer to "a scan that produces nothing is read as clean" — the class this file
# was the third measured instance of. See scripts/lib/scan-guard.sh for the three instances and for why
# it lives under scripts/lib/ rather than in sandbox/probes/text.sh.
. "$root/scripts/lib/scan-guard.sh"

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

# Shell code spliced into the next run BEFORE common.sh is sourced, and reset to the stub below after it.
# `stage` runs after the source and is enough for anything a STEP needs; this exists for what common.sh
# reads AT SOURCE TIME — the agent's user, home and PATH, and the `sudo` its pre-flight calls.
#
# EVERY RUN GETS THE STUB, because common.sh has no degraded mode: a run whose `sudo` cannot reach the
# agent user is refused at the pre-flight. The stub is a `sudo` that strips `-n -u <user> --`, then runs
# the rest under an EMPTY environment plus a fixed secure PATH — modelling `env_reset` (G4, see THE
# ENVIRONMENT ACROSS THE SWITCH in common.sh) so a variable `probe_as_agent`'s own `env ...` line forgets
# to pass does not come back through inheritance — with PROBE_AGENT_USER pointed at the user running the
# suite and PROBE_AGENT_PATH at a value the suite's own PATH cannot already contain. That is not the
# boundary — one uid runs everything — but it is the real `probe_as_agent` argument vector AND the real
# environment discipline in front of the real command.
#
# THE STUB MUST BE THE `sudo` THAT RUNS, and the preamble refuses to source common.sh otherwise. Windows 11
# ships a real `sudo.exe` in System32, on a Git Bash PATH, with different flags and a UAC elevation; and
# PROBE_AGENT_USER below names a user that DOES resolve. So if writing the stub ever failed, common.sh's
# pre-flight would call that program for real. `command -v` only looks the name up; it runs nothing, so
# the check itself cannot reach it. Exit 97 is this refusal's own status, distinct from every probe's.
privsep_preamble=$(cat <<'PREAMBLE'
cat > "$PROBE_OUT/bin/sudo" <<'STUB'
#!/bin/sh
# Enough of sudo for probe_as_agent: drop the flags it passes, then run what follows at the SAME uid but
# under an EMPTY environment plus a fixed secure PATH — modelling `env_reset` + `secure_path`, so anything
# not passed explicitly through the `env ...` that follows does not come back by inheritance.
while [ $# -gt 0 ]; do
    case $1 in
        -n) shift ;;
        -u) shift 2 ;;
        --) shift; break ;;
        *) break ;;
    esac
done
exec env -i PATH=/usr/bin:/bin "$@"
STUB
chmod +x "$PROBE_OUT/bin/sudo"
if [ "$(command -v sudo)" != "$PROBE_OUT/bin/sudo" ]; then
    echo "suite: 'sudo' resolves to '$(command -v sudo)', not the stub; refusing to source common.sh" >&2
    exit 97
fi
PROBE_AGENT_USER=$(id -un)
PROBE_AGENT_HOME=$PROBE_OUT/agent-home
# A marker segment the inherited PATH cannot already contain (a directory under this run's own $PROBE_OUT),
# so a `PATH=...` dropped from probe_as_agent's `env` line is observable rather than silently identical to
# the harness's own PATH.
PROBE_AGENT_PATH=$PROBE_OUT/agent-path-marker:$PATH
PREAMBLE
)
preamble=$privsep_preamble

# A probe script, written to a temporary directory and run exactly as the container runs one: from the
# repository root, because that is where `sandbox/run.sh`'s `copy=` assignment leaves a probe and it is
# how common.sh resolves its own sibling files.
run_probe() {
    out=$(mktemp -d)
    script=$(mktemp)
    cat > "$script" <<SCRIPT
cd $root
PROBE_OUT=$out
PROBE_STATE=$out/state
PROBE_TARGET=$out/target
PROBE_TIMEOUT=5
PATH=$out/bin:\$PATH
mkdir -p "\$PROBE_OUT/bin"
$preamble
. sandbox/probes/common.sh
$1
SCRIPT
    preamble=$privsep_preamble
    mkdir -p "$out/bin"
    set +e
    sh "$script" > "$out/stdout" 2>&1
    probe_status=$?
    set -e
    # What `sandbox/run.sh`'s `eng cp` block does once the container has stopped: lift `failures` and
    # `steps` out of the state directory — which the measured party cannot reach, so it cannot forge them
    # — into the results directory, which is where `transcript.sh`'s `exit-codes:` block reads `steps`.
    # A scratch state path, never the container's real /home/probe/.probe-state: this suite runs on the
    # host. Emulating the copy here is what lets every check below keep reading $PROBE_OUT_DIR.
    cp -R "$out/state/." "$out/" 2>/dev/null || true
    PROBE_OUT_DIR=$out
}

# fake_agent <name> <body>: an executable on the probe's PATH, so a step can be tested without an agent.
# Call it after run_probe has made the directory, and before the run that uses it — see `staged` below.
staged=

# stage <name> <body>: queue an executable to be created inside the next run_probe, at $PROBE_OUT/bin.
stage() {
    staged="$staged
mkdir -p \"\$PROBE_OUT/bin\"
cat > \"\$PROBE_OUT/bin/$1\" <<'BODY'
#!/bin/sh
$2
BODY
chmod +x \"\$PROBE_OUT/bin/$1\""
}

run_staged() {
    run_probe "$staged
$1"
    staged=
}

# --- where the harness writes -----------------------------------------------------------------------
#
# THE DEFECT CLASS: every artefact in $PROBE_OUT is written by a `>` the harness performs, and `>` follows
# a symlink. While $PROBE_OUT was `/out` — a host mount `run.sh`'s docker branch has to make
# world-writable, inside a container the measured party shares — the agent never had to write one of those
# files. It only had to create the PATH first, as a link to $PROBE_STATE/failures, and the harness's own
# redirect did the rest. Measured end-to-end in a container: an empty `delta-<label>.txt` (the label is
# deterministic and public) truncated `failures` to 0 bytes and `probe_finish` exited 0 with a failed step
# already recorded. `rm -f` before each redirect does not close it — shell has no `O_NOFOLLOW`, so the
# link comes back between the unlink and the open — and a per-redirect guard is the wrong shape when a
# dozen redirects share the defect. The directory moved instead, which is what these two checks pin.
#
# Asserted against the SOURCE, not by running the harness: every run below overrides PROBE_OUT to a
# scratch directory, so the DEFAULT — the one value the container uses — is the one thing no run can see.
probe_out_default=$(sed -n 's/^PROBE_OUT=${PROBE_OUT:-\(.*\)}$/\1/p' "$root/sandbox/probes/common.sh")
check "the harness's output directory is not the /out bind mount" \
    "$probe_out_default" "/home/probe/.probe-out"

# THE TWIN, and unpinned until now for no reason but that nobody wrote it. $PROBE_STATE is overridden by
# `run_probe` exactly as $PROBE_OUT is, so its default is equally invisible to every check in this file:
# MEASURED, changing common.sh's PROBE_STATE default leaves this suite byte-identical and green.
#
# What it costs in a container is worse than the artefacts, because this directory holds the run's STATUS.
# `run.sh`'s `eng cp` block lifts `/home/probe/.probe-state` by that literal path, so a default pointing
# anywhere else strands `failures` and `steps` inside the container.
#
# THE CONSEQUENCE CHANGED, and this paragraph used to describe the old one: it said the assembler would
# print `exit-codes: (not recorded)` and the verifier would accept a transcript recording no step as a
# clean run. That is no longer possible — an absent step list is now itself a refusal. What a stranded
# `steps` costs today is the opposite and still a defect: a COMPLETE measurement is discarded, because the
# run looks to the assembler exactly like one the harness refused. Either way the default must be what
# `run.sh` lifts, which is what this pins.
probe_state_default=$(sed -n 's/^PROBE_STATE=${PROBE_STATE:-\(.*\)}$/\1/p' "$root/sandbox/probes/common.sh")
check "the harness's bookkeeping directory is where run.sh lifts it from" \
    "$probe_state_default" "/home/probe/.probe-state"

# THE THIRD OF THE TRIO, and the one with no pin at all until now. $PROBE_OUT's and $PROBE_STATE's
# defaults above are literal paths a `sed` capture can compare directly; $PROBE_TARGET's default is
# `$PROBE_AGENT_HOME/probe-target` (an expression, not a literal), so the captured text is evaluated
# with PROBE_AGENT_HOME pinned to ITS OWN sed-extracted default before the comparison. `run.sh`'s
# `cleanup` lifts `/home/agent/probe-target` by that literal path (see run.sh's `eng cp` block for the
# profile directory), so a default pointing anywhere else strands the profile directory the agent was
# pointed at inside the removed container, exactly as an unpinned $PROBE_STATE would have stranded
# `failures` and `steps`.
probe_agent_home_default=$(sed -n 's/^PROBE_AGENT_HOME=${PROBE_AGENT_HOME:-\(.*\)}$/\1/p' "$root/sandbox/probes/common.sh")
probe_target_expr=$(sed -n 's/^PROBE_TARGET=${PROBE_TARGET:-\(.*\)}$/\1/p' "$root/sandbox/probes/common.sh")
probe_target_default=$(
    # shellcheck disable=SC2034 # read by the `eval` below, not referenced directly -- shellcheck
    # cannot see through eval's re-parse of $probe_target_expr, which is where $PROBE_AGENT_HOME expands.
    PROBE_AGENT_HOME=$probe_agent_home_default
    eval "printf %s \"$probe_target_expr\""
)
check "the profile directory's default is where run.sh lifts it from" \
    "$probe_target_default" "/home/agent/probe-target"

# --- run.sh's copy-back, RUN rather than grepped ----------------------------------------------------
#
# THE DEFECT CLASS: a source-text assertion wearing a behavioural name. This used to be
# `grep -c '^eng cp "$id:/home/probe/.probe-out/." "$out/"' run.sh` equal to 1, called "run.sh lifts the
# harness's output directory out of the stopped container" — and a mutant that wrapped that line in
# `if [ "$mode" = shell ]; then ... fi`, keeping it at column 0, left it GREEN while no probe run got any
# artefact out at all: `transcript.sh` emitted `(not recorded)` for install, version, help, strings and
# every delta. A `grep` is indifferent to control flow, so the check could not fail for the reason its
# name gave. It now RUNS `sandbox/run.sh` against a stub engine, which needs no Docker and no network, and
# asserts what the name says: the artefacts arrive in the results directory, and they arrive before the
# container is removed.
#
# fake_engine_run <interrupt>: run `sandbox/run.sh probe claude` with a stub `docker` first on the PATH.
# A non-empty argument makes the stub's `run` send SIGINT to `run.sh` itself, which is what Ctrl-C during a
# multi-minute agent install does. Sets $eng_status, $eng_order (the cp/rm/rmi calls in order) and
# $eng_lifted (the marker files the stub's `cp` left behind, which stand in for the copied directories).
fake_engine_run() {
    eng_dir=$(mktemp -d)
    cat > "$eng_dir/docker" <<'ENGINE'
#!/bin/sh
# Records every subcommand, and for `cp` drops one recognisable file where the real engine would have put
# the container directory named in "$2" (`<id>:/path/to/dir/.`).
#
# The full argument vector goes to a SECOND file, because the first is matched line-by-line as bare
# subcommands and widening it would break that. The target matters for `diff`: the image tag and the
# container name were once the same string, and `diff` is the only subcommand here that accepts either.
printf '%s\n' "$1" >> "$FAKE_ENG_LOG"
printf '%s\n' "$*" >> "$FAKE_ENG_LOG.args"
case $1 in
    run)
        if [ -n "${FAKE_ENG_INTERRUPT:-}" ]; then
            kill -INT "$PPID"
            # WAIT FOR THE PARENT TO BE GONE, not for a duration. This was `sleep 5`, and that raced: the
            # stub must not return before the signal has been acted on, but under load the parent can be
            # descheduled past any fixed window -- the sleep then expires, this returns normally, run.sh
            # carries on, and the run exits 0 instead of 130. MEASURED by a review seat: the check failed
            # twice in full-suite runs while three agents were working, and passed 3/3 with the machine
            # idle. The bound is kept so a parent that never dies fails the check instead of hanging the
            # suite, and the poll costs nothing when idle because it ends the moment the parent is gone.
            probe_eng_waited=0
            while kill -0 "$PPID" 2>/dev/null; do
                probe_eng_waited=$((probe_eng_waited + 1))
                [ "$probe_eng_waited" -gt 600 ] && break
                sleep 0.1 2>/dev/null || sleep 1
            done
        fi
        ;;
    inspect) echo 0 ;;
    cp)
        mkdir -p "$3"
        echo copied > "$3/from-$(basename "$(dirname "${2#*:}")")"
        ;;
    diff) echo "stub diff" ;;
esac
exit 0
ENGINE
    chmod +x "$eng_dir/docker"
    : > "$eng_dir/calls"
    # The results directory is found by listing, not by reading the "results in ..." line `cleanup` prints:
    # measured on this host, an interrupted `sh` writes NOTHING more to its inherited stderr even though the
    # trap runs to the end (an `echo` marker spliced into `cleanup` past that line fired on both paths while
    # both streams came back zero bytes). A test that read the message would pass only where that quirk is
    # absent, which is the opposite of what it is for.
    mkdir -p "$root/target/sandbox"
    : > "$eng_dir/before"
    for d in "$root"/target/sandbox/*/; do
        if [ -d "$d" ]; then printf '%s\n' "$d" >> "$eng_dir/before"; fi
    done
    set +e
    FAKE_ENG_LOG="$eng_dir/calls" FAKE_ENG_INTERRUPT="$1" AGENT_PROFILE_SANDBOX_ENGINE=docker \
        PATH="$eng_dir:$PATH" sh "$root/sandbox/run.sh" probe claude > "$eng_dir/stdout" 2>&1
    eng_status=$?
    set -e
    eng_new=
    for d in "$root"/target/sandbox/*/; do
        if [ -d "$d" ] && ! grep -qxF "$d" "$eng_dir/before"; then eng_new=$d; fi
    done
    eng_order=$(grep -xE 'cp|rm|rmi' "$eng_dir/calls" | tr '\n' ',')
    # The TARGET of each call, not just its name. `eng diff` takes an image OR a container, so a run that
    # named both the same string could ask the engine about the wrong one and never notice.
    eng_diff_target=$(sed -n 's/^diff //p' "$eng_dir/calls.args" | head -n1)
    eng_cp_target=$(sed -n 's/^cp \([^:]*\):.*/\1/p' "$eng_dir/calls.args" | head -n1)
    eng_build_target=$(sed -n 's/^build --tag \([^ ]*\) .*/\1/p' "$eng_dir/calls.args" | head -n1)
    # Never let an empty name through to the `cd` and the `rm -rf` below. Say so as a failed check rather
    # than as a green one: a run that produced no results directory at all is exactly the outcome these
    # checks exist to notice.
    if [ -z "$eng_new" ]; then
        eng_lifted='(the stub engine run left no results directory)'
        rm -rf "$eng_dir"
        return 0
    fi
    eng_out=${eng_new%/}
    eng_lifted=$( (cd "$eng_out" && find . -name 'from-*' | sort | tr '\n' ',') || true)
    rm -rf "$eng_dir" "$eng_out"
}

fake_engine_run ''
check "run.sh lifts the harness's output directory out of the stopped container" \
    "$eng_lifted" "./from-.probe-out,./from-.probe-state,./target/from-probe-target,"
check "and lifts it before the container is removed" "$eng_order" "cp,cp,cp,rm,rmi,"

# THE ENGINE MUST BE ASKED ABOUT THE CONTAINER, NOT THE IMAGE. `eng diff` is the only subcommand here that
# accepts either, and the two were once the same string -- so the question "what did the RUN write" was
# answered with "what the IMAGE BUILD wrote". MEASURED on a real probe run: `diff.txt` came back holding
# this Containerfile's final rustup layer and not one path the container had touched, while the agent's own
# installed binary was absent from it. That block is the only instrument for a write to a location nobody
# predicted, and it reported the wrong filesystem in silence.
check "the engine is asked what the CONTAINER changed, not the image" \
    "$([ "$eng_diff_target" = "$eng_build_target" ] && echo same-as-image || echo distinct)" "distinct"
# And the copy-back must read that same container, or the artefacts come from somewhere else again.
check "and the artefacts are copied out of the container the diff asked about" \
    "$([ "$eng_cp_target" = "$eng_diff_target" ] && echo same || echo different)" "same"

# A SIGINT during a multi-minute install used to destroy every artefact the run had bought: the copies were
# on the main path, `exit 130` went straight to the EXIT trap, and `eng rm -f` took the container away with
# all of it inside. Measured here against the stub before the copies moved into `cleanup`: the call log read
# `build, run, rm, rmi` with no `cp` at all, and the results directory held build.log and an empty
# output.log. It failed CLOSED — `$out/exit-code` is not written on that path either, so `transcript.sh`
# reads `probe_exit=unknown` rather than assembling a transcript from half a run — so this is evidence
# durability, not correctness. The evidence is still what the run was for.
fake_engine_run interrupt
check "an interrupted run exits 130" "$eng_status" "130"
check "and still lifts its artefacts out before the container is removed" \
    "$eng_lifted" "./from-.probe-out,./from-.probe-state,./target/from-probe-target,"
check "and the removal still happens, after the copies" "$eng_order" "cp,cp,cp,rm,rmi,"

# --- run.sh's own refusals --------------------------------------------------------------------------
#
# `fake_engine_run` above always calls `sandbox/run.sh probe claude` with a valid agent and no version —
# every other arm of run.sh's own argument handling is unexercised: the version-charset guard, the
# agent-charset guard and the missing-probe-script refusal. The version guard is the serious one: it
# stands between a probe's version argument and the harness-uid shell that assembles the container's
# command line. Without it, `sandbox/run.sh probe claude "1.0'; id; echo PWNED; :'"` splices `id` and
# `echo PWNED` into that shell as separate commands, running BEFORE any install, at the uid running the
# harness, with every recorded artefact still writable.
#
# fake_engine_case <arg...>: run `sandbox/run.sh` with a stub `docker` and the given arguments. Sets
# $case_status, $case_stderr (stderr only) and $case_ran (1 if the stub's `run` subcommand was ever
# invoked). $case_ran is what makes a refusal check non-vacuous: a guard that fired too LATE would still
# exit non-zero, but only "no run" says the injected content never reached the container's command line.
fake_engine_case() {
    case_dir=$(mktemp -d)
    cat > "$case_dir/docker" <<'ENGINE'
#!/bin/sh
printf '%s\n' "$1" >> "$FAKE_ENG_LOG"
case $1 in
    inspect) echo 0 ;;
    diff) echo "stub diff" ;;
esac
exit 0
ENGINE
    chmod +x "$case_dir/docker"
    : > "$case_dir/calls"
    set +e
    case_stderr=$(FAKE_ENG_LOG="$case_dir/calls" AGENT_PROFILE_SANDBOX_ENGINE=docker \
        PATH="$case_dir:$PATH" sh "$root/sandbox/run.sh" "$@" 2>&1 >/dev/null)
    case_status=$?
    set -e
    if grep -qx run "$case_dir/calls"; then case_ran=1; else case_ran=0; fi
    rm -rf "$case_dir"
}

fake_engine_case probe claude '1.0; id'
check "run.sh refuses a version outside the charset before it reaches the container (semicolon)" \
    "$case_status" "2"
check "and names the charset (semicolon)" "$case_stderr" "sandbox: versions are [A-Za-z0-9._-]"
check "and the container is never told to run (semicolon)" "$case_ran" "0"

fake_engine_case probe claude '1.0 --flag'
check "run.sh refuses a version outside the charset before it reaches the container (space)" \
    "$case_status" "2"
check "and names the charset (space)" "$case_stderr" "sandbox: versions are [A-Za-z0-9._-]"
check "and the container is never told to run (space)" "$case_ran" "0"

fake_engine_case probe claude '$(id)'
check 'run.sh refuses a version outside the charset before it reaches the container (command substitution)' \
    "$case_status" "2"
check 'and names the charset (command substitution)' "$case_stderr" "sandbox: versions are [A-Za-z0-9._-]"
check 'and the container is never told to run (command substitution)' "$case_ran" "0"

fake_engine_case probe 'claude;id'
check "run.sh refuses an agent word outside [a-z0-9-]" "$case_status" "2"
check "and names the agent charset" "$case_stderr" "sandbox: agent names are [a-z0-9-]"
check "and the container is never told to run (bad agent word)" "$case_ran" "0"

fake_engine_case probe nosuchagent
check "run.sh refuses an agent with no probe script" "$case_status" "2"
check "and names the missing script" "$case_stderr" \
    "sandbox: no probe script sandbox/probes/nosuchagent.sh"
check "and the container is never told to run (no probe script)" "$case_ran" "0"

# --- probe_record ---------------------------------------------------------------------------------

run_probe 'probe_record ok true'
check "a probe whose steps all pass exits zero" "$probe_status" "0"

run_probe 'probe_record boom sh -c "exit 3"'
check "a probe whose only step failed exits non-zero" "$probe_status" "1"
check "the failing step's exit code is recorded" "$(cat "$PROBE_OUT_DIR/boom.exit-code")" "3"

# The regression that matters: a failure in the MIDDLE must neither abort the probe nor be forgotten by it.
run_probe 'probe_record first sh -c "exit 3"
probe_record second true
probe_record third true'
check "a middle failure still exits non-zero" "$probe_status" "1"
check "steps after a failure are still recorded" "$(cat "$PROBE_OUT_DIR/third.exit-code")" "0"

# `steps` is what the transcript's exit-codes block is built from, in the order the probe ran them.
run_probe 'probe_record one true
probe_record two sh -c "exit 3"
probe_record three true'
check "every step is logged once, in order" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" "one 0,two 3,three 0,"
check "the command is recorded beside its output" \
    "$(cat "$PROBE_OUT_DIR/two.cmd")" "sh -c exit 3"

# A step that fails for its OWN reason must land in the log exactly once too. Routing probe_record's
# failure path through probe_fail appended it twice, which reads as the probe having run it twice.
run_probe 'probe_record ok true
probe_strings nosuchagent SOME_KEY'
check "a non-command failure is logged once" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" "ok 0,strings 127,"

# `failures` and `steps` do not live in $PROBE_OUT, and this is what pins the separation. When they did,
# /out was a read-write host mount beside `/src:ro` that `run.sh`'s docker branch made world-writable, and
# the agent — or the install scripts `npm install --global` runs before the agent exists — shared that
# directory at the same uid: truncating `failures` made `probe_finish` exit 0 and the matrix job go green,
# and §5.3's byte-comparison could not tell, because artifact and committed file both derive from the
# forged bytes. $PROBE_OUT is now container-internal and probe-owned too, so the stub below is no longer a
# thing the agent could run. What the check still pins is the SEPARATION itself: nothing that writes an
# ARTEFACT can name the RUN'S STATUS by accident — which the planted delta symlink shows is not the same
# as being out of reach, since it aimed a $PROBE_OUT redirect straight across the gap.
run_probe 'probe_record boom sh -c "exit 3"
: > "$PROBE_OUT/failures"
printf "boom 0\nafter 0\n" > "$PROBE_OUT/steps"
probe_record after true'
check "forging /out's bookkeeping does not make a failed probe exit zero" "$probe_status" "1"
check "the copy-back replaces the forged steps log with the real one" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" "boom 3,after 0,"

run_probe 'probe_record slow sleep 30'
check "a hanging step is killed and recorded" "$(cat "$PROBE_OUT_DIR/slow.exit-code")" "124"
check "a hanging step fails the probe" "$probe_status" "1"

# --- the privilege boundary: refused when absent, and the switch's argument vector ------------------
#
# READ THIS BEFORE TRUSTING THE GREEN. The container runs the harness as `probe` and everything measured
# as `agent`, and that split is the only thing that makes the bookkeeping above tamper-proof rather than
# merely moved. THIS SUITE DOES NOT EXERCISE THE UID SEPARATION. It runs on the maintainer's host, where
# there is no `agent` user, and every run goes through the stub `sudo` defined at the top of this file,
# which runs the command at the SAME uid. Nothing below asserts that `agent` cannot write `failures`; only
# a real container run does, and a guarantee no test covers must not read as though one does.
#
# EXACTLY WHAT IS COVERED, because an earlier version of this paragraph over-promised and one of the
# checks under it was measured vacuous:
#
#   that a run with no usable boundary is REFUSED at common.sh's pre-flight — before the first crossing,
#   so before any agent code runs — and recorded as a `privilege` failure. There used to be a degraded
#   mode instead, in which every crossing silently became a direct call at the harness's uid and the run
#   produced a transcript indistinguishable from a real one;
#
#   that the plumbing `probe_as_agent` puts in front of a command does not reach the recorded command.
#   This was once asserted with the switch compiled out, where `probe_as_agent` prepended nothing at all:
#   a seat mutated `probe_record` to record what actually ran — `sudo -n -u agent -- env HOME=... sh -c
#   exit 3` into the `.cmd` text — and every privilege check here stayed GREEN. There was nothing to leak,
#   so the check could not fail for the reason its name gave, while in a container the same mutant writes
#   harness internals into a COMMITTED transcript as though they were evidence about the agent.

# THE REFUSAL. A user that does not exist is the "no agent user in the image" case. The observable that
# matters is not the exit status alone but that NOTHING CROSSED: `probe_make_target`, common.sh's first
# crossing, runs immediately after the pre-flight, so a target directory that exists means the run got
# past the gate. The step below it must never run either.
preamble=$(printf '%s\n' "$privsep_preamble" | sed 's/^PROBE_AGENT_USER=.*/PROBE_AGENT_USER=probe-suite-no-such-user/')
run_probe 'probe_record_agent never true'
check "a run whose agent user does not exist is refused" "$probe_status" "1"
check "and the refusal is recorded as a privilege failure" \
    "$(cat "$PROBE_OUT_DIR/state/failures")" "harness-privilege 1"
check "and nothing crossed the boundary before the refusal" \
    "$([ -e "$PROBE_OUT_DIR/target" ] && echo crossed || echo none)" "none"
check "and no step after it ran" \
    "$([ -e "$PROBE_OUT_DIR/never.exit-code" ] && echo ran || echo none)" "none"

# The "sudo present but the rule does not reach the agent" case: a stub that refuses everything.
preamble=$(printf '%s\n' "$privsep_preamble" | sed 's/^exec env -i PATH=\/usr\/bin:\/bin "\$@"$/exit 1/')
run_probe 'probe_record_agent never true'
check "a run whose sudo cannot reach the agent user is refused" "$probe_status" "1"
check "and that refusal is recorded as a privilege failure too" \
    "$(cat "$PROBE_OUT_DIR/state/failures")" "harness-privilege 1"
check "and nothing crossed before that refusal either" \
    "$([ -e "$PROBE_OUT_DIR/target" ] && echo crossed || echo none)" "none"
# "No sudo on the PATH at all" is the third branch and is not staged here, deliberately: both hosts this
# suite runs on carry a real `sudo` on their standard PATH (Windows System32; `/usr/bin` on the Linux
# runner), so removing it means hiding the directories the rest of common.sh needs. It is one `command -v`
# test in front of the same refusal the two cases above reach.

# --- the exit-97 guard's REJECTING half -------------------------------------------------------------
#
# `privsep_preamble`'s own `if [ "$(command -v sudo)" != "$PROBE_OUT/bin/sudo" ]; then ... exit 97; fi` is
# what stands between this suite and calling the host's REAL `sudo` — Windows 11's `sudo.exe`, which
# elevates through UAC — should the stub above it ever fail to land where `sudo` resolves. Every check in
# this file exercises its ACCEPTING half: the stub always does resolve, so the guard never fires. Deleting
# the `exit 97` line entirely would leave this suite exactly as green as it is now, because nothing runs
# the guard with the stub actually absent from PATH.
#
# Derived from `privsep_preamble`, as the two refusal checks above derive theirs, rather than a second copy
# of the stub — and safely, by construction rather than luck:
#   (a) the stub is written to `$PROBE_OUT/sudo-unused` — outside `$PROBE_OUT/bin`, the only directory
#       `run_probe`'s script puts on PATH — so `command -v sudo` cannot resolve to it. That is "the stub is
#       absent from PATH" for the guard's purposes, without touching the host's real PATH or its sudo.
#   (b) `PROBE_AGENT_USER` names a user that does not exist. This is what makes a mutant that deletes the
#       guard's `exit 97` safe to run: if the guard were ever broken, execution would fall through into
#       common.sh's own pre-flight, which checks `command -v sudo`, then `id "$PROBE_AGENT_USER"` — and
#       refuses at the `id` step, status 1, BEFORE it ever reaches `sudo -n -u "$PROBE_AGENT_USER" true`. So
#       even a broken guard, combined with (b), can never execute a `sudo` invocation that could elevate —
#       real or stub.
preamble_misplaced_stub=$(printf '%s\n' "$privsep_preamble" \
    | sed -e 's|$PROBE_OUT/bin/sudo|$PROBE_OUT/sudo-unused|g' \
          -e 's/^PROBE_AGENT_USER=.*/PROBE_AGENT_USER=probe-suite-no-such-user/')
preamble=$preamble_misplaced_stub
run_probe ''
check "the guard refuses a run whose stub does not resolve as sudo" "$probe_status" "97"

# The transcript states what the VENDOR documents. `sudo -n -u agent -- env HOME=... npm install ...`
# states that plus a fact about this harness, and only the first is evidence about the agent.
run_probe 'probe_record_agent boom sh -c "exit 3"
probe_record_agent crossed sh -c "printf %s \"\$HOME\""
command -v sudo > "$PROBE_OUT/sudo-used"'
# THE CONTROL. Without these two the check below is the vacuous one again: a switch that did not go
# through the stub would leave the recorded command clean for the wrong reason.
check "the stub is the sudo the switch goes through" \
    "$(cat "$PROBE_OUT_DIR/sudo-used")" "$PROBE_OUT_DIR/bin/sudo"
check "and the command really is run through it" \
    "$(cat "$PROBE_OUT_DIR/crossed.txt")" "$PROBE_OUT_DIR/agent-home"
check "the privilege switch is not written into the recorded command" \
    "$(cat "$PROBE_OUT_DIR/boom.cmd")" "sh -c exit 3"
check "a failed agent-side step still fails the probe" "$probe_status" "1"

# --- G4: the environment survives the switch only because probe_as_agent passes it explicitly ---------
#
# THE ARGUMENT VECTOR ALONE IS NOT THE CONTRACT. common.sh's "THE ENVIRONMENT ACROSS THE SWITCH (G4)"
# states that `sudo` resets the environment, so HOME, PATH and NPM_CONFIG_PREFIX survive only because
# `probe_as_agent` passes them explicitly through `env`. A stub that only stripped `sudo`'s flags and exec'd
# the rest at the same uid would still show HOME/PATH/NPM_CONFIG_PREFIX correctly even if one of those three
# were dropped from `probe_as_agent` — the SAME uid's own inherited copy would silently stand in. The stub
# above now resets the environment first (an empty environment plus a fixed secure PATH), and
# PROBE_AGENT_PATH is a value the inherited PATH cannot already contain, so a dropped assignment is
# observable rather than indistinguishable from the harness's own environment.
preamble=$privsep_preamble
run_probe 'probe_record_agent g4home sh -c "printf %s \"\$HOME\""
probe_record_agent g4path sh -c "printf %s \"\$PATH\""
probe_record_agent g4npm sh -c "printf %s \"\$NPM_CONFIG_PREFIX\""
command -v sudo > "$PROBE_OUT/sudo-used"'
check "the stub is the sudo the switch goes through, for the checks below" \
    "$(cat "$PROBE_OUT_DIR/sudo-used")" "$PROBE_OUT_DIR/bin/sudo"
check "HOME crosses the switch because probe_as_agent passes it explicitly" \
    "$(cat "$PROBE_OUT_DIR/g4home.txt")" "$PROBE_OUT_DIR/agent-home"
check "PATH crosses the switch because probe_as_agent passes it explicitly" \
    "$(cat "$PROBE_OUT_DIR/g4path.txt")" "$PROBE_OUT_DIR/agent-path-marker:$PROBE_OUT_DIR/bin:$PATH"
check "NPM_CONFIG_PREFIX crosses the switch because probe_as_agent passes it explicitly" \
    "$(cat "$PROBE_OUT_DIR/g4npm.txt")" "$PROBE_OUT_DIR/agent-home/.local"

# --- the privilege enumeration is complete -----------------------------------------------------------
#
# common.sh's THE PRIVILEGE SPLIT block lists, per helper, which side of the boundary it runs on. That
# enumeration IS the boundary — a `probe_as_agent` added to a helper nobody listed, or dropped from one
# that needs it, changes which uid runs the code and nothing downstream notices — and it is hand-written
# prose asserting a mechanical property, which is the shape that rots. Three separate reviews have now
# found a false sentence in it; the third found the sentence claiming a wrapper on `probe_version`'s
# extraction "would fail outright", which was wrong because `>` binds to the CALLING shell.
#
# THE MEMBERSHIP HALF IS MECHANICAL FROM HERE. Every function in common.sh whose body calls
# `probe_as_agent` or `probe_record_agent` must be NAMED in the block, so a new crossing cannot be added
# in silence. WHAT THIS CANNOT CATCH, stated because a green check that reads as more than it is caused
# the defect above: it cannot tell whether the REASON written beside a name is true, and a false reason is
# exactly what all three reviews found. It also cannot see a crossing added to a probe script rather than
# to common.sh, and it matches a name anywhere in the block, so a name that appears only inside someone
# else's rationale would satisfy it. It is a completeness check, not a correctness one.
#
# It lives here rather than in scripts/ because this suite already runs in `just check` and in CI and
# already makes assertions against common.sh's source; a new script would need the justfile and the CI
# workflow changed to be anything but dead code.
split_block=$(awk '/^# WHICH HELPER RUNS AS WHICH USER/, /^# THE ENVIRONMENT ACROSS THE SWITCH/' \
    "$root/sandbox/probes/common.sh")
crossing_fns=$(awk '
    /^[A-Za-z_][A-Za-z0-9_]*\(\)[[:space:]]*\{$/ { fn = $1; sub(/\(\).*/, "", fn); next }
    /^}$/ { fn = ""; next }
    /^[[:space:]]*#/ { next }
    /probe_as_agent|probe_record_agent|(^|[;&|(!]|then|else|do|exec)[[:space:]]*sudo([[:space:]]|$)/ { if (fn != "") print fn }
' "$root/sandbox/probes/common.sh" | LC_ALL=C sort -u)
unlisted=
for fn in $crossing_fns; do
    printf '%s\n' "$split_block" | grep -qE "$fn([^A-Za-z0-9_]|\$)" || unlisted="$unlisted $fn"
done
check "every function that crosses the privilege boundary is named in the enumeration" "$unlisted" ""
# The control: a collector that found nothing would pass the check above without a word. The fourteen are
# probe_make_target, probe_record, probe_npm_install, probe_uv_install, probe_script_install,
# probe_version, probe_help, probe_strings, probe_pristine, probe_snapshot, probe_prepare_target,
# probe_restore_default, probe_apply, probe_as_agent — the last one not because it CALLS
# probe_as_agent/probe_record_agent, but because its own body is the
# `sudo -n -u "$PROBE_AGENT_USER" -- env` invocation the other thirteen wrap.
check "and the enumeration is checked against a non-empty list of them" \
    "$(printf '%s\n' "$crossing_fns" | grep -c .)" "14"

# R6-1: NAME-membership is blind to a crossing added INSIDE a function that already crosses. Wrapping
# `probe_as_agent` onto probe_strings' `find` — which common.sh calls "the single most attractive mistake
# in this file", because it would run agent-chosen paths at the harness uid — leaves the collected NAME
# unchanged, the membership check above satisfied, and the count-of-12 control untouched: everything above
# stays green. So pin the number of crossing LINES per function, not just whether it crosses at all.
#
# F2: the same blindness recurs one level down. Both collectors used to match only `probe_as_agent` and
# `probe_record_agent` by NAME, so a crossing spelled as a bare `sudo` — prefixing probe_strings' `find`
# with `sudo -n -u "$PROBE_AGENT_USER" --`, the mistake common.sh itself calls the single most attractive
# one in the file — left every check above green. The pattern below now counts a line that INVOKES sudo
# (at the start of a line, or after `;`, `&`, `|`, `(`, `!`, `then`, `else`, `do` or `exec`, followed by
# whitespace or end of line), not a line that merely mentions the word: `command -v sudo` and the two
# messages naming it in the pre-flight below do not match. It also counts top-level lines — outside every
# function — under the pseudo-name `<top>`, because a crossing planted above the first function common.sh
# defines was invisible to a collector that only ever tracked `fn`.
#
# Mirrors the same function-boundary and comment-skipping rules as the crossing_fns collector above (kept
# as a second pass rather than folded into it, because this needs a running COUNT per function rather than
# the set of functions with a nonzero one) — if those three patterns ever change, change them in both
# places. Table measured at HEAD by counting non-comment lines matching this pattern inside each function
# body of common.sh, plus the top-level lines outside every function.
#
# A LEGITIMATE change to a crossing must update this table deliberately, and that update IS the point: the
# count forces a reviewer to look at the added or removed line, which name-membership cannot.
#
# WHAT THIS PATTERN CANNOT CATCH, stated because three rounds of widening it each implied it was complete.
# It is a HEURISTIC FOR ATTRIBUTION -- it answers "which function, how many" -- and it is not a shell
# parser. MEASURED misses: `if sudo`, `while sudo`, `until sudo`, `eval sudo`, a backtick substitution,
# `command sudo`, `env sudo`, `xargs sudo`, `time sudo`, `{ sudo ...; }`, assignment then `"$CMD"`, and
# `"$(command -v sudo)"` -- which misses the SUFFIX anchor too, because `)` follows the word directly, and
# is the most reachable of them since `command -v` is already this file's own idiom in probe_strings.
#
# The PINNED LINE SETS below are what covers that. They cannot say WHICH FUNCTION a line belongs to, and
# they do not care how the call is spelled: any line mentioning the word, written any way, must appear in
# the set verbatim. Anchored table and pinned text cover each other's blind spot; neither does alone.
#
# This was a pair of COUNTS first, and the counts were unsound: three of the five `sudo` lines are prose,
# so one commit that reworded a message string and added a crossing netted to zero and left the suite
# byte-identical to its baseline. A total over a set containing editable prose is not a tripwire.
crossing_counts=$(awk '
    /^[A-Za-z_][A-Za-z0-9_]*\(\)[[:space:]]*\{$/ { fn = $1; sub(/\(\).*/, "", fn); count = 0; next }
    /^}$/ { if (fn != "" && count > 0) print fn, count; fn = ""; next }
    /^[[:space:]]*#/ { next }
    /probe_as_agent|probe_record_agent|(^|[;&|(!]|then|else|do|exec)[[:space:]]*sudo([[:space:]]|$)/ {
        if (fn != "") count++; else top++
    }
    END { if (top > 0) print "<top>", top }
' "$root/sandbox/probes/common.sh" | LC_ALL=C sort)
crossing_counts_expected=$(cat <<'TABLE'
<top> 1
probe_apply 4
probe_as_agent 1
probe_help 1
probe_make_target 1
probe_npm_install 1
probe_prepare_target 2
probe_pristine 1
probe_record 1
probe_restore_default 4
probe_script_install 1
probe_snapshot 4
probe_strings 1
probe_uv_install 1
probe_version 1
TABLE
)
check "each crossing function's line count is pinned against the table measured at HEAD, not just its name" \
    "$crossing_counts" "$crossing_counts_expected"

# THE SPELLING-BLIND HALF, and it pins the LINES rather than how many there are.
#
# A COUNT WAS TRIED HERE FIRST AND WAS UNSOUND. It totalled the non-comment lines mentioning the word, and
# three of the five are prose -- `command -v sudo` and two message strings -- which are the most
# edit-prone lines in the file. MEASURED: one commit that rewords a message string and adds the
# `if sudo ... -- find` crossing nets to zero, and the whole suite stayed byte-identical to its baseline
# with a real crossing present. A total over a set that contains editable prose is not a tripwire.
#
# The pinned TEXT has neither failure. A reworded line changes the list, so nothing can net out; and the
# repair is to read the line the diff shows, not to edit a digit -- a number invites being bumped, which
# would codify the very blind spot the anchored pattern above already has.
#
# Scoped across EVERY probe script, not just common.sh: the twelve source it, and a crossing written in
# one of them was read by nothing here.
sudo_mentions=$(for probe_file in "$root"/sandbox/probes/*.sh; do
    grep -vE '^[[:space:]]*#' "$probe_file" \
        | grep -E '(^|[^A-Za-z0-9_])sudo([^A-Za-z0-9_]|$)' \
        | sed "s|^[[:space:]]*|$(basename "$probe_file")  |"
done | LC_ALL=C sort)
sudo_mentions_expected=$(cat <<'MENTIONS'
common.sh  elif ! sudo -n -u "$PROBE_AGENT_USER" true >/dev/null 2>&1; then
common.sh  if ! command -v sudo >/dev/null 2>&1; then
common.sh  probe_privilege_refusal="'sudo -n -u $PROBE_AGENT_USER true' failed"
common.sh  probe_privilege_refusal="there is no 'sudo' on the PATH"
common.sh  sudo -n -u "$PROBE_AGENT_USER" -- env \
MENTIONS
)
check "every line under sandbox/probes that mentions sudo is one of these, whatever the spelling" \
    "$sudo_mentions" "$sudo_mentions_expected"

# The flag decides WHICH UID a command runs at, so a value left set after a call would silently put the
# next step — a snapshot, a restore, anything the harness does for itself — on the wrong side.
run_probe 'probe_record_agent one true
printf "[%s]\n" "$PROBE_AS_AGENT" > "$PROBE_OUT/flag"'
check "the agent flag does not leak past the call that set it" \
    "$(cat "$PROBE_OUT_DIR/flag")" "[]"

# --- which failures the harness claims as its OWN ---------------------------------------------------
#
# The `harness-` prefix is what `sandbox/transcript.sh` keys on to refuse assembling a committable
# transcript, so the SET of names carrying it is a contract, not a naming style. A new harness-side
# failure added without the prefix silently reopens the defect: the run gets a transcript named
# `<id>-unknown.md`, indistinguishable from a genuine "this agent would not install" measurement, and
# §5.3's verifier accepts it against a failed job.
#
# Pinned as a whole-table string, like the crossing counts above, so BOTH directions fail loudly: a new
# unprefixed harness failure, and a prefix put on a failure that is really the agent's. The three that are
# deliberately NOT prefixed are in this table too, which is the half a "does every harness failure have
# the prefix?" check could not express -- `pristine-N` and `restore-N` are the harness failing to archive
# or restore, but the vendor's install code runs first and can cause exactly that, and `strings 127` is
# the agent's binary missing from its own PATH.
probe_fail_names=$(grep '^[[:space:]]*probe_fail ' "$root/sandbox/probes/common.sh" \
    | sed 's/^[[:space:]]*probe_fail //' | awk '{print $1}' | LC_ALL=C sort -u)
probe_fail_names_expected=$(cat <<'NAMES'
"harness-mechanism-$probe_name"
"pristine-$probe_slot"
"restore-$probe_rn"
harness-privilege
harness-version-refused
strings
NAMES
)
check "every failure the harness records is classified, and the classification has not drifted" \
    "$probe_fail_names" "$probe_fail_names_expected"
# The control: a collector that found nothing would satisfy the check above without a word.
check "and that table was checked against a non-empty collection" \
    "$(printf '%s\n' "$probe_fail_names" | grep -c .)" "6"
# AND THE SPELLING-BLIND HALF, for the same reason as the crossing counts above. The collector one line up
# requires `probe_fail` to be the FIRST TOKEN ON ITS LINE, so `[ -n "$x" ] && probe_fail harness-thing 1`
# is invisible to it -- MEASURED: adding exactly that leaves the collected table byte-identical and this
# check prints ok. A one-liner is a more idiomatic refactor of the multi-line blocks in `common.sh` than
# what is written there now, so this is not a contrived spelling.
#
# That blindness is how the whole `harness-` mechanism unravels: an unseen call site is one nobody is
# forced to look at, so nobody notices it lacks the prefix, so `transcript.sh` assembles a committable
# transcript for a run the harness refused.
#
# PINNED AS TEXT, and scoped across every probe script, for the three reasons a count failed here. A count
# of LINES cannot see two calls joined with `;` on one line -- the text can, because the line itself
# changes. A count reading only common.sh cannot see a `probe_fail` in one of the twelve, which is a
# helper they all source -- MEASURED: one added to `codex.sh` left the whole suite green while its
# unprefixed name would have let `transcript.sh` assemble evidence for a harness-side refusal. And a
# number invites being bumped, where a line has to be read before it can be pasted in.
probe_fail_lines=$(for probe_file in "$root"/sandbox/probes/*.sh; do
    grep -vE '^[[:space:]]*#' "$probe_file" \
        | grep -E '(^|[^A-Za-z0-9_])probe_fail([^A-Za-z0-9_]|$)' \
        | sed "s|^[[:space:]]*|$(basename "$probe_file")  |"
done | LC_ALL=C sort)
probe_fail_lines_expected=$(cat <<'CALLS'
common.sh  probe_fail "harness-mechanism-$probe_name" 2
common.sh  probe_fail "pristine-$probe_slot" 1
common.sh  probe_fail "restore-$probe_rn" 1
common.sh  probe_fail harness-privilege 1
common.sh  probe_fail harness-version-refused 2
common.sh  probe_fail strings 127
common.sh  probe_fail() {
CALLS
)
check "every line under sandbox/probes that names probe_fail is one of these" \
    "$probe_fail_lines" "$probe_fail_lines_expected"

# THE OTHER WRITER OF `steps`, which three rounds of guards never bounded. `transcript.sh` refuses a run
# whose step list carries a `harness-` row, and `steps` is written from TWO places: `probe_fail` above,
# and `probe_record`/`probe_record_agent`. Everything above reads only the first. MEASURED: a step named
# `harness-help` through `probe_record_agent` makes the assembler discard a fully successful run --
# `probe-exit: 0`, a real version -- and write no transcript at all. The failure direction is the safe one
# (evidence lost, not forged), but the comment above calls this set a contract, and half of it was unheld.
#
# Only LITERAL names can be checked here; `probe_apply`'s are built from a mechanism label at runtime, and
# those labels come from the four known prefixes, none of which can begin `harness-`.
record_names=$(for probe_file in "$root"/sandbox/probes/*.sh; do
    grep -vE '^[[:space:]]*#' "$probe_file" \
        | grep -oE 'probe_record(_agent)?[[:space:]]+[A-Za-z0-9_-]+' \
        | sed 's/.*[[:space:]]//'
done | LC_ALL=C sort -u)
check "no literal step name given to probe_record claims the harness's own prefix" \
    "$(printf '%s\n' "$record_names" | grep -c '^harness-' || true)" "0"
check "and that was checked against the literal names actually found" \
    "$record_names" "$(printf 'help\ninstall\nservice-stop-dir\nservice-stop-file\nversion\n')"

# --- the agent's home ---------------------------------------------------------------------------
#
# The twelve probe scripts name the agent's default locations through PROBE_AGENT_HOME, because under two
# users `$HOME` is the HARNESS's home and not where the agent writes. Its default is `/home/agent`, and
# there is no longer a second default: it used to fall back to the harness's own home whenever the
# boundary was absent, and that branch was the site of the worst defect this harness has had — every probe
# watching the harness's home while the agent wrote its own, producing the empty delta `probe_delta` calls
# a launch that changed nothing, unambiguously.
#
# Asserted WITHOUT the preamble's explicit override — stripped from the shared preamble rather than written
# out again, so the two cannot drift apart. Without that strip the check would only re-read a value the
# test itself had set.
preamble=$(printf '%s\n' "$privsep_preamble" | grep -v '^PROBE_AGENT_HOME=')
run_probe 'printf "%s\n" "$PROBE_AGENT_HOME" > "$PROBE_OUT/agent-home"'
check "the agent's home defaults to the second user's, not the harness's" \
    "$(cat "$PROBE_OUT_DIR/agent-home")" "/home/agent"
# And the default does not quietly become the harness's home when this suite's own HOME is in play — the
# old fallback's exact shape. Compared as inequality so it fails only for the defect it names.
check "and that default is not the harness's own home" \
    "$([ "$(cat "$PROBE_OUT_DIR/agent-home")" = "$HOME" ] && echo harness-home || echo distinct)" "distinct"

# --- probe_behaviour ------------------------------------------------------------------------------

# THE CONTAINERFILE'S TRUSTED-BINARY LIST IS TIED TO THE CODE, not maintained beside it. The absolute
# paths below are only a defence while the binaries they name are ones the agent cannot replace, and the
# image build asserts exactly that. A hand-maintained list in a different file from the thing it protects
# is the shape that rotted the privilege-split enumeration three times, so the list is derived here and
# compared, and a sixth absolute path added to `common.sh` without updating the build turns this red.
#
# It reads the SOURCE, not a running container: this suite has no container, and what it is checking is
# an agreement between two files in the repository.
trusted_used=$(grep -oE 'probe_as_agent /usr/bin/[a-z]+' "$root/sandbox/probes/common.sh" \
    | sed 's|.*/||' | LC_ALL=C sort -u | tr '\n' ' ')
trusted_asserted=$(grep -oE '^    for probe_bin in [a-z ]+; do' "$root/sandbox/Containerfile" \
    | sed -E 's/^    for probe_bin in //; s/; do$//' | tr ' ' '\n' | grep -v '^$' \
    | LC_ALL=C sort -u | tr '\n' ' ')
check "the image asserts exactly the binaries the harness invokes by absolute path" \
    "$trusted_asserted" "$trusted_used"
# The control: two empty strings compare equal, so a regex that matched nothing on BOTH sides would pass
# the check above while proving nothing at all -- and either spelling could drift out from under it.
check "and that comparison was made against a non-empty set" \
    "$(printf '%s' "$trusted_used" | wc -w | tr -d ' ')" "5"

# THE SNAPSHOT MUST NOT BE HIJACKABLE BY THE MEASURED PARTY. `probe_snapshot` reads as the agent, because
# a shared group does not survive mode 0700 and a harness-side read of such a directory silently reports
# it as empty. Reading as the agent is only safe while the BINARY doing the reading is one the agent
# cannot replace: `probe_as_agent` hands the command a PATH whose first element is
# `$PROBE_AGENT_HOME/.local/bin`, which the agent owns because NPM_CONFIG_PREFIX points npm's global
# installs there. MEASURED in a container: a `find` planted there IS executed by that idiom.
#
# So this plants a `find` on the PATH the probe runs with and asserts the snapshot is unaffected. If the
# call ever loses its absolute path, the planted binary answers instead and the measured party gets to
# decide what the measurement says — which is the whole threat the two-uid split exists to prevent.
stage find 'echo "HIJACKED"'
run_staged 'mkdir -p "$PROBE_OUT/watched" && echo real > "$PROBE_OUT/watched/evidence.txt"
probe_snapshot hijack "$PROBE_OUT/watched"'
check "a find planted on the agent PATH does not supply the snapshot" \
    "$(grep -c HIJACKED "$PROBE_OUT_DIR/hijack.txt" || true)" "0"
check "and the real find still recorded the file it was supposed to see" \
    "$(grep -c 'evidence.txt$' "$PROBE_OUT_DIR/hijack.txt" || true)" "1"
# The control: without it, a snapshot that silently recorded NOTHING would satisfy the check above just as
# well as one that correctly used /usr/bin/find, and the test would pass over a broken measurement.
check "and that fixture really did stage a hijacking find on the PATH" \
    "$(PATH="$PROBE_OUT_DIR/bin:$PATH" find "$PROBE_OUT_DIR/watched" 2>/dev/null | grep -c HIJACKED || true)" "1"

# probe_behaviour must baseline the target as well as the default location: agent-profile and the probe
# itself create files there, and counting them as the agent's would read as isolation that did not happen.
run_probe 'mkdir -p "$PROBE_TARGET" && echo ours > "$PROBE_TARGET/pre-existing"
probe_behaviour true /nonexistent-default env:SOME_HOME @none'
check "the behaviour step records a baseline" \
    "$([ -f "$PROBE_OUT_DIR/baseline-env-SOME_HOME.txt" ] && echo yes || echo no)" "yes"
check "the baseline covers the default location too" \
    "$(grep -c '^# ' "$PROBE_OUT_DIR/baseline-env-SOME_HOME.txt")" "2"

# THE assertion that separates "the agent's config moved" a from "the probe created it". The probe makes
# the target directory itself and, for a file mechanism, writes the candidate file — so a post-launch
# LISTING always contains the probe's own bytes. Reading that as the agent's work yields a false
# ConfigIsolation: Supported carrying a `measured:` basis, and every gate still passes, because the gates
# check a claim's shape and not whether the measurement meant anything.
stage quiet 'exit 0'
run_staged 'probe_behaviour quiet /nonexistent-default flagfile:--config "{}"'
check "a target populated before the launch is not in the delta" \
    "$(cat "$PROBE_OUT_DIR/delta-flagfile---config.txt")" ""
check "the candidate the PROBE wrote is in the baseline" \
    "$(grep -c 'config$' "$PROBE_OUT_DIR/baseline-flagfile---config.txt")" "1"

# The positive control, without which the check above passes for a delta that is always empty.
stage noisy 'mkdir -p "$2"/sub 2>/dev/null || true; : > "$(dirname "$2")/written-by-the-agent"'
run_staged 'probe_behaviour noisy /nonexistent-default flagfile:--config "{}"'
check "a file the agent wrote IS in the delta" \
    "$(grep -Fxc "+ f $PROBE_OUT_DIR/target/written-by-the-agent" "$PROBE_OUT_DIR/delta-flagfile---config.txt")" "1"

# The fixtures above all watch /nonexistent-default, so only one watched location could ever contribute a
# row - the ambiguity the bug produces was structurally unreachable. These two use a REAL second location.

# The defect: `%P`/`%f` strip the location, so a file the agent wrote into the target and one it wrote into
# its default location - same basename - are byte-identical delta rows. Full paths must tell them apart.
default_loc_1=$(mktemp -d)
dualwriter_body=': > "'"$default_loc_1"'/state.json"; : > "$(dirname "$2")/state.json"'
stage dualwriter "$dualwriter_body"
run_staged "probe_behaviour dualwriter $default_loc_1 flagfile:--config \"{}\""
check "the delta says which location a file landed in" \
    "$(grep -Fxc "+ f $PROBE_OUT_DIR/target/state.json" "$PROBE_OUT_DIR/delta-flagfile---config.txt") $(grep -Fxc "+ f $default_loc_1/state.json" "$PROBE_OUT_DIR/delta-flagfile---config.txt")" \
    "1 1"

# The defect: a config that MOVED out of the default location into the target - the strongest possible
# isolation evidence - appears on both sides of `comm` under a bare basename and cancels to an empty delta.
default_loc_2=$(mktemp -d)
: > "$default_loc_2/config"
mover_body='mv "'"$default_loc_2"'/config" "$(dirname "$2")/config"'
stage mover "$mover_body"
run_staged "probe_behaviour mover $default_loc_2 flagfile:--config @none"
check "a config that moved out of the default location shows as both a plus and a minus" \
    "$(grep -Fxc -e "+ f $PROBE_OUT_DIR/target/config" "$PROBE_OUT_DIR/delta-flagfile---config.txt") $(grep -Fxc -e "- f $default_loc_2/config" "$PROBE_OUT_DIR/delta-flagfile---config.txt")" \
    "1 1"

# Each of the four mechanism kinds must reach the agent as the SHAPE it expects. Two kinds cannot express
# OpenCode's OPENCODE_CONFIG (a variable naming a file) or Cline's --data-dir (a flag naming a directory),
# and pointing either at the wrong shape makes the agent's refusal look like a failure to isolate.
stage recorder 'echo "argv: $*"; echo "SOME_VAR=${SOME_VAR-unset}"'
run_staged 'probe_behaviour recorder /nonexistent-default env:SOME_VAR @none'
check "env: sets the variable to the profile directory" \
    "$(sed -n 2p "$PROBE_OUT_DIR/behaviour-env-SOME_VAR.txt")" "SOME_VAR=$PROBE_OUT_DIR/target"

stage recorder 'echo "argv: $*"; echo "SOME_VAR=${SOME_VAR-unset}"'
run_staged 'probe_behaviour recorder /nonexistent-default envfile:SOME_VAR "{}"'
check "envfile: sets the variable to a file inside it" \
    "$(sed -n 2p "$PROBE_OUT_DIR/behaviour-envfile-SOME_VAR.txt")" "SOME_VAR=$PROBE_OUT_DIR/target/config"
check "envfile: writes the candidate to that file" \
    "$(cat "$PROBE_OUT_DIR/target/config")" "{}"

stage recorder 'echo "argv: $*"'
run_staged 'probe_behaviour recorder /nonexistent-default flagdir:--data-dir @none'
check "flagdir: passes the profile directory" \
    "$(cat "$PROBE_OUT_DIR/behaviour-flagdir---data-dir.txt")" \
    "argv: --data-dir $PROBE_OUT_DIR/target --version"

stage recorder 'echo "argv: $*"'
run_staged 'probe_behaviour recorder /nonexistent-default flagfile:--config "{}"'
check "flagfile: passes a file inside it" \
    "$(cat "$PROBE_OUT_DIR/behaviour-flagfile---config.txt")" \
    "argv: --config $PROBE_OUT_DIR/target/config --version"

# `--version` is a floor: an agent that exits before initialising writes nothing, and an empty delta then
# reads as "the mechanism moves nothing". A probe that knows a better command must be able to say so.
stage recorder 'echo "argv: $*"'
run_staged 'probe_behaviour recorder /nonexistent-default env:SOME_VAR @none --print hello'
check "a probe can choose the command that makes its agent initialise" \
    "$(cat "$PROBE_OUT_DIR/behaviour-env-SOME_VAR.txt")" "argv: --print hello"

run_probe 'probe_behaviour true /nonexistent-default bogus:THING @none'
check "an unknown mechanism fails the probe" "$probe_status" "1"

# --- probe_candidates -----------------------------------------------------------------------------

# §8.4: the smallest content the agent accepts is part of the mechanism, and Aider proved guessing costs —
# missing, empty and comment-only each exit 2 while `{}` is accepted (aider.rs's `INITIAL_CONFIG` constant).
stage picky 'test -s "$2" || exit 2; grep -q "{}" "$2" || exit 2'
run_staged 'probe_candidates picky flagfile:--config @none "" "# comment" "{}"'
check "a missing candidate file is recorded as refused" \
    "$(cat "$PROBE_OUT_DIR/candidate-1.exit-code")" "2"
check "an empty candidate is recorded as refused" \
    "$(cat "$PROBE_OUT_DIR/candidate-2.exit-code")" "2"
check "a comment-only candidate is recorded as refused" \
    "$(cat "$PROBE_OUT_DIR/candidate-3.exit-code")" "2"
check "the accepted candidate is recorded as accepted" \
    "$(cat "$PROBE_OUT_DIR/candidate-4.exit-code")" "0"
check "the index says what each candidate held" \
    "$(sed -n 4p "$PROBE_OUT_DIR/candidates.txt")" "4: {}"

# The defect: every one of those three refusals used to land in `failures`, so the probe exited non-zero,
# so the matrix job concluded failure, so `verify-transcripts.sh` refused the transcript — and aider, amp
# and continue, whose sweeps are DESIGNED to produce refusals, could not produce a committable transcript
# at all. A refusal is the measurement §8.4 asks for, not an error.
check "a sweep of refused candidates does not fail the probe" "$probe_status" "0"
check "but every refusal is still recorded, in order, with its code" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" \
    "candidate-1 2,candidate-2 2,candidate-3 2,candidate-4 0,"
check "and none of them is recorded as a failure" \
    "$(wc -l < "$PROBE_OUT_DIR/failures" | tr -d ' ')" "0"

# The regression the fix above could introduce, and the reason `PROBE_SOFT` is scoped to the sweep's own
# launch: a probe that sweeps candidates must not thereby go green when the step that MEASURES ISOLATION
# failed. Here the sweep accepts `{}` and refuses @none, and the behaviour step is then launched against a
# target the stub rejects.
stage picky 'test -s "$2" || exit 2; grep -q "{}" "$2" || exit 2'
run_staged 'probe_candidates picky flagfile:--config "{}" @none
probe_behaviour picky /nonexistent-default flagfile:--config @none'
check "a failed behaviour step still fails a probe that swept candidates" "$probe_status" "1"
check "and the sweep's refusal is not what failed it" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/failures")" "behaviour-flagfile---config 2,"
check "while the sweep's refusal is still in the step log" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" \
    "candidate-1 0,candidate-2 2,behaviour-flagfile---config 2,"

# --- PROBE_EXPECT ---------------------------------------------------------------------------------
#
# For some agents the ONLY command that initialises is one that fails. Measured: `pi` writes nothing at
# all under `--version`, `list` or `auth check`, and writes its whole configuration under `-p hello`,
# which then exits 1 because no API key is set. That is a successful measurement, and the first real
# probe run recorded it as a failed probe: the job failed and `verify-transcripts.sh` accepts a
# transcript only when `probe-exit` is 0, so the evidence the launch existed to produce was refused.
stage refuser 'echo "no API key"; exit 1'
run_staged 'PROBE_EXPECT=1
probe_behaviour refuser /nonexistent-default env:REFUSER_HOME @none'
check "a behaviour step that exits the code its probe declared does not fail the probe" \
    "$probe_status" "0"
check "and the declared non-zero code is still recorded in the step log" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/steps")" "behaviour-env-REFUSER_HOME 1,"
check "and it is not recorded as a failure" \
    "$(wc -l < "$PROBE_OUT_DIR/failures" | tr -d ' ')" "0"

# THE HALF THAT DISTINGUISHES THIS FROM `PROBE_SOFT`, and the only half worth having. Soft tolerates ANY
# non-zero code, which would turn a segfault, a missing shared library or a timeout into a successful
# measurement with an empty delta. A declaration names ONE code, and every other code -- including 0 --
# is still a failure. Without this check the two are indistinguishable: everything above would pass just
# as well against a `PROBE_SOFT` widened to behaviour steps.
stage compliant 'exit 0'
run_staged 'PROBE_EXPECT=1
probe_behaviour compliant /nonexistent-default env:COMPLIANT_HOME @none'
check "a behaviour step that exits 0 where non-zero was declared FAILS the probe" \
    "$probe_status" "1"
check "and the unexpected success is what is recorded as the failure" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/failures")" "behaviour-env-COMPLIANT_HOME 0,"

# A declaration that outlived its launch would silently excuse the NEXT step, and the next step is
# routinely a second mechanism's launch whose own expected code is 0. `probe_behaviour` clears it, so the
# second launch below is judged against 0 and fails on its non-zero exit even though the first declared 1.
stage twofailed 'exit 1'
run_staged 'PROBE_EXPECT=1
probe_behaviour twofailed /nonexistent-default env:FIRST_HOME @none
probe_behaviour twofailed /nonexistent-default env:SECOND_HOME @none'
check "a declaration does not leak past the launch it was made for" \
    "$(tr "\n" "," < "$PROBE_OUT_DIR/failures")" "behaviour-env-SECOND_HOME 1,"

# `probe_candidates`' own loop restores every registered location (`probe_restore_known`) BEFORE each
# candidate's launch, so the sweep runs every candidate against the same default location the one before
# it saw — not just the first, which is all the checks above happen to exercise since none of their
# stubs write to a registered default location at all. This stub exits 0 the first time it finds ITS OWN
# marker already sitting in the default location, and 2 otherwise, and plants that marker on every
# launch: a restored location is empty going in, so a working sweep sees "no marker" (2) every time,
# while a sweep that stopped restoring between candidates would see the first launch's marker on every
# launch after it (2,0,0).
default_loc_sweep=$(mktemp -d)
rm -rf "$default_loc_sweep"
sweep_body='marker="'"$default_loc_sweep"'/marker"
mkdir -p "'"$default_loc_sweep"'"
if [ -e "$marker" ]; then result=0; else result=2; fi
: > "$marker"
exit "$result"'
stage sweepagent "$sweep_body"
run_staged "probe_pristine $default_loc_sweep
probe_candidates sweepagent flagfile:--config @none @none @none"
check "each candidate in a sweep is launched against the same default location as the one before it" \
    "$(cat "$PROBE_OUT_DIR/candidate-1.exit-code") $(cat "$PROBE_OUT_DIR/candidate-2.exit-code") $(cat "$PROBE_OUT_DIR/candidate-3.exit-code")" \
    "2 2 2"

# --- the default location, between launches -------------------------------------------------------

# The defect: nothing reset the agent's DEFAULT location between launches, so only the FIRST launch of a
# probe was attributable. `probe_candidates` launches once per candidate and each launch initialises that
# location; the `probe_behaviour` that followed then baselined over the files the sweep had already
# written, and they sat on both sides of `comm` and cancelled out of the delta — which reads as the
# mechanism isolating, the exact wrong answer. Measured before the fix, with this stub: 0 bytes.
default_loc_3=$(mktemp -d)
rm -rf "$default_loc_3"
repeater_body='mkdir -p "'"$default_loc_3"'"; : > "'"$default_loc_3"'/config.json"'
stage repeater "$repeater_body"
run_staged "probe_pristine $default_loc_3
probe_candidates repeater flagfile:--config \"{}\" \"{}\"
probe_behaviour repeater $default_loc_3 flagfile:--config \"{}\""
check "a launch after an earlier launch is still attributable" \
    "$(grep -Fxc "+ f $default_loc_3/config.json" "$PROBE_OUT_DIR/delta-flagfile---config.txt")" "1"

# The same defect where reordering cannot reach it: measuring cline's three mechanisms or opencode's two
# means launching three times or twice, and every launch after the first was blind. Nothing here calls
# `probe_pristine` — the registration has to happen on its own, the first time `probe_behaviour` is handed
# the location — because the nine probe scripts that only measure behaviour say nothing about it.
default_loc_4=$(mktemp -d)
rm -rf "$default_loc_4"
rewriter_body='mkdir -p "'"$default_loc_4"'"; : > "'"$default_loc_4"'/config.json"'
stage rewriter "$rewriter_body"
run_staged "probe_behaviour rewriter $default_loc_4 env:SOME_HOME @none
probe_behaviour rewriter $default_loc_4 flagdir:--data-dir @none"
check "a second mechanism's delta is not blinded by the first mechanism's launch" \
    "$(grep -Fxc "+ f $default_loc_4/config.json" "$PROBE_OUT_DIR/delta-flagdir---data-dir.txt")" "1"

# A default location is not always a directory — Aider's is the file `.aider.conf.yml` (aider.rs's
# `FILE_NAME` constant) — and it is not always the agent CREATING something: this stub DELETES its own
# config, which is why the restore is an exact unpack of the pre-launch state rather than "remove whatever
# appeared". A heuristic that only undoes additions leaves the second launch with nothing left to delete,
# and the `- f` row that says the agent removed its config never appears again.
default_file=$(mktemp)
: > "$default_file"
deleter_body='rm -f "'"$default_file"'"'
stage deleter "$deleter_body"
run_staged "probe_behaviour deleter $default_file env:SOME_HOME @none
probe_behaviour deleter $default_file flagdir:--data-dir @none"
check "a file default location is restored between launches" \
    "$(grep -Fxc -e "- f $default_file" "$PROBE_OUT_DIR/delta-flagdir---data-dir.txt")" "1"

# --- probe_strip_ansi and probe_excerpt -----------------------------------------------------------

run_probe 'printf "\033[1;31mred\033[0m plain\n" | probe_strip_ansi > "$PROBE_OUT/stripped"'
check "colour sequences are removed" "$(cat "$PROBE_OUT_DIR/stripped")" "red plain"

run_probe 'printf "a\007b\r\n" | probe_strip_ansi | od -c | head -n1 > "$PROBE_OUT/stripped"'
check "stray control bytes are removed" \
    "$(awk '{print $2 $3 $4}' "$PROBE_OUT_DIR/stripped")" "ab\n"

run_probe 'printf "1\n2\n3\n4\n" | probe_excerpt 2 > "$PROBE_OUT/excerpt"'
check "an excerpt is bounded" "$(head -n2 "$PROBE_OUT_DIR/excerpt" | tr "\n" " ")" "1 2 "
check "a truncated excerpt says so at the cut" \
    "$(sed -n 3p "$PROBE_OUT_DIR/excerpt")" "  [truncated: more than 2 lines]"

run_probe 'printf "1\n2\n" | probe_excerpt 5 > "$PROBE_OUT/excerpt"'
check "an excerpt that fits carries no marker" "$(grep -c truncated "$PROBE_OUT_DIR/excerpt")" "0"

# probe_excerpt's OTHER bound: a single line over 500 characters is cut and the cut is marked with
# "  [line truncated]" — only the line-COUNT bound above was ever asserted; the per-line bound was not.
probe_excerpt_600=$(awk 'BEGIN { s = ""; for (i = 0; i < 600; i++) s = s "x"; print s }')
probe_excerpt_500=$(awk 'BEGIN { s = ""; for (i = 0; i < 500; i++) s = s "x"; print s }')

run_probe "printf '%s\n' '$probe_excerpt_600' | probe_excerpt 5 > \"\$PROBE_OUT/excerpt\""
check "an over-long line is cut and says so at the cut" \
    "$(cat "$PROBE_OUT_DIR/excerpt")" "$probe_excerpt_500  [line truncated]"

# The boundary control: without it, the check above would pass just as well for a filter that truncates
# every line to 500 regardless of its own length.
run_probe "printf '%s\n' '$probe_excerpt_500' | probe_excerpt 5 > \"\$PROBE_OUT/excerpt\""
check "a line at the bound is passed whole" "$(cat "$PROBE_OUT_DIR/excerpt")" "$probe_excerpt_500"

# --- probe_version --------------------------------------------------------------------------------

stage fakeagent 'echo "fakeagent 1.2.3-beta (build 77)"'
run_staged 'probe_version fakeagent'
check "the version is the first digit-bearing run" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "1.2.3-beta"
check "the raw version output is kept verbatim" \
    "$(cat "$PROBE_OUT_DIR/version.txt")" "fakeagent 1.2.3-beta (build 77)"

# Gate A accepts `0m` — it is inside the charset it checks — so an un-stripped colour reset would be
# recorded as the version, name an evidence file, and pass every gate. This is the check that pins it.
stage fakeagent 'printf "\033[0m\033[1mfakeagent\033[0m 4.5.6\n"'
run_staged 'probe_version fakeagent'
check "a coloured banner does not become the version" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "4.5.6"

stage fakeagent 'echo "no version here"'
run_staged 'probe_version fakeagent'
check "an agent that prints no version records unknown" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "unknown"

stage fakeagent 'echo "boom" >&2; exit 4'
run_staged 'probe_version fakeagent'
check "a failing --version fails the probe" "$probe_status" "1"
check "a failing --version with no digit anywhere records unknown" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "unknown"

# The defect: the extraction used to take the first digit-bearing token in the WHOLE capture, and
# `probe_record` folds stderr into that same capture (`2>&1`). Ordinary npm/python startup noise on
# stderr then out-races the real version line. Measured with stubs that print what real CLIs print:
# `(node:1234) [DEP0040] DeprecationWarning: ...` then `fakeagent 2.7.1` extracted `1234`, and a python
# traceback path then `fakeagent 0.86.2` extracted `python3.12`. Both sit inside Gate A's charset.
stage fakeagent 'echo "(node:1234) [DEP0040] DeprecationWarning: '"'"'punycode'"'"' is deprecated" >&2
echo "fakeagent 2.7.1"'
run_staged 'probe_version fakeagent'
check "a node deprecation warning on stderr does not become the version" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "2.7.1"

stage fakeagent 'echo "/usr/lib/python3.12/site-packages/x.py:41: SyntaxWarning: invalid escape" >&2
echo "fakeagent 0.86.2"'
run_staged 'probe_version fakeagent'
check "a python warning path on stderr does not become the version" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "0.86.2"

# The rejected wrong fix: extracting from the LAST non-empty line alone breaks here, because an npm
# update-notifier banner prints AFTER the version line. The executable-name preference has to run first.
stage fakeagent 'echo "fakeagent 3.1.0"
echo "Update available 3.1.0 -> 4.0.0"'
run_staged 'probe_version fakeagent'
check "an update-notifier banner after the version does not override it" \
    "$(cat "$PROBE_OUT_DIR/version.extracted")" "3.1.0"

# --- probe_strings --------------------------------------------------------------------------------

stage fakeagent 'echo "reads FAKE_API_KEY and FAKE_HOME"'
run_staged 'probe_strings fakeagent FAKE_API_KEY MISSING_TOKEN'
check "a documented variable that is present is recorded" \
    "$(grep -c '^documented FAKE_API_KEY: present$' "$PROBE_OUT_DIR/strings.txt")" "1"
check "a documented variable that is absent is recorded" \
    "$(grep -c '^documented MISSING_TOKEN: ABSENT$' "$PROBE_OUT_DIR/strings.txt")" "1"
# The undocumented sweep is the half that pays: FAKE_HOME was never passed in.
check "an undocumented variable is still found" \
    "$(grep -c '^FAKE_HOME$' "$PROBE_OUT_DIR/strings.txt")" "1"

# A LARGE SINGLE BINARY IS STILL SCANNED. `probe_strings` caps the files it reads at 64M so a vendored
# toolchain beside the entry point cannot turn this step into the job's time limit, and that cap used to
# apply to the resolved executable as well. MEASURED against `@opencode/cli`, which ships one 200529376-byte
# executable and nothing else in its `bin` directory: the scan matched NO file, and every documented
# variable was recorded ABSENT while a control grep proved the names are in the binary. The failure is
# silent and it reads as a finding, which is the worst of both -- an adapter would conclude the agent
# supports no isolation mechanism at all.
#
# The fixture is padded PAST the cap rather than merely near it, so the check fails if the exemption is
# removed or the cap is merely raised to some larger arbitrary number. `truncate` extends sparsely, so the
# file costs no disk; `probe_strings` never executes the staged agent, it only resolves and greps it, so
# the padding cannot affect anything but the size.
stage bigagent 'echo "reads BIG_API_KEY"'
run_staged 'truncate -s 65M "$PROBE_OUT/bin/bigagent"
probe_strings bigagent BIG_API_KEY'
check "a resolved executable larger than the scan cap is still read" \
    "$(grep -c '^documented BIG_API_KEY: present$' "$PROBE_OUT_DIR/strings.txt")" "1"
# The control: without it the check above passes for a fixture that was never actually over the cap, which
# is the one way this test could certify the exemption while proving nothing about it.
check "and that fixture really was over the cap" \
    "$(find "$PROBE_OUT_DIR/bin" -maxdepth 1 -type f -name bigagent -size -64M | wc -l)" "0"

# The undocumented sweep's filter is eleven alternatives
# (KEY|TOKEN|SECRET|CREDENTIAL|PASSWORD|AUTH|HOME|CONFIG|DATA_DIR|PROFILE|SETTINGS), and until now only
# one of them (HOME, via FAKE_HOME above) was ever exercised. This is the half that found Claude's
# `CLAUDE_CODE_USE_BEDROCK` and three OAuth variables — one token per category, plus a negative control
# in the SAME fixture: a token of the same [A-Z][A-Z0-9_]{3,} shape that names no category, so a filter
# that passed everything through unfiltered could not satisfy this check either.
stage categoryagent 'echo "X_KEY X_TOKEN X_SECRET X_CREDENTIAL X_PASSWORD X_AUTH X_HOME X_CONFIG X_DATA_DIR X_PROFILE X_SETTINGS UNRELATED_BANNER"'
run_staged 'probe_strings categoryagent'
# `if ... ; then` rather than `grep ... && printf ...`: the latter, inside this command substitution
# and under `set -eu`, aborts the WHOLE suite the moment a category is absent -- exactly what a narrowed
# filter does to most of these eleven -- instead of letting the check below report a clean FAIL. An `if`
# condition is the POSIX-exempted form; a bare command on the left of `&&` here is not.
probe_strings_found=$(for probe_cat_tok in X_KEY X_TOKEN X_SECRET X_CREDENTIAL X_PASSWORD X_AUTH \
    X_HOME X_CONFIG X_DATA_DIR X_PROFILE X_SETTINGS; do
    if grep -qx "$probe_cat_tok" "$PROBE_OUT_DIR/strings.txt"; then printf '%s ' "$probe_cat_tok"; fi
done)
check "the undocumented sweep reports a token for every category the filter claims" \
    "$probe_strings_found" \
    "X_KEY X_TOKEN X_SECRET X_CREDENTIAL X_PASSWORD X_AUTH X_HOME X_CONFIG X_DATA_DIR X_PROFILE X_SETTINGS "
check "and a token in no category is not reported" \
    "$(grep -c '^UNRELATED_BANNER$' "$PROBE_OUT_DIR/strings.txt")" "0"

run_probe 'probe_strings nosuchagent SOME_KEY'
check "scanning a missing executable fails the probe" "$probe_status" "1"
check "scanning a missing executable records an exit code" \
    "$(cat "$PROBE_OUT_DIR/strings.exit-code")" "127"

# --- the version passthrough ----------------------------------------------------------------------

# A vendor script takes whatever version it takes. Dropping the request silently would put a version the
# installer never saw into a transcript that §5.3 then commits byte-for-byte.
run_probe 'PROBE_VERSION=9.9.9
probe_script_install https://example.invalid/install.sh'
check "an installer that cannot pin refuses a version" "$probe_status" "2"
check "the refusal is recorded under the harness's own name, not the agent's install" \
    "$(cat "$PROBE_OUT_DIR/harness-version-refused.exit-code")" "2"
# The other half, and the reason the name changed: this refusal is the OPERATOR asking for something the
# installer cannot do, so it must not land in the row a genuine vendor install failure writes. Sharing
# `install` made the two indistinguishable in a transcript's `exit-codes:` block.
check "and nothing is recorded as an install failure" \
    "$([ -e "$PROBE_OUT_DIR/install.exit-code" ] && echo recorded || echo none)" "none"

# --- the scan guard itself -------------------------------------------------------------------------
#
# scripts/lib/scan-guard.sh is the repository's one answer to "a scan that produces nothing is read as
# clean", and it is the thing every check below now rests on, so its REJECTING half is tested here and not
# only its accepting half. A guard that cannot refuse is the exact failure it exists to stop.
#
# It is tested HERE for the reason written beside the privilege-enumeration check above: this suite already
# runs in `just check` and in CI and already asserts against source files outside itself, while a new test
# script would need the justfile and the CI workflow changed to be anything but dead code.
scan_dir=$(mktemp -d)
: > "$scan_dir/one.sh"
: > "$scan_dir/two.sh"
scan_list=$(mktemp)

scan_expand "$scan_list" must-find 'the fixture' "$scan_dir"/*.sh
check "an expansion that matched keeps every path that exists" \
    "$(tr '\n' ',' < "$scan_list")" "$scan_dir/one.sh,$scan_dir/two.sh,"

# THE CLASS, as small as it goes: the directory the glob names is not there, so POSIX sh hands the loop
# the pattern's own text. The literal word must not become an input, and the empty list must not be a pass.
if scan_expand "$scan_list" must-find 'the fixture' "$scan_dir"/nowhere/*.sh 2>/dev/null; then
    scan_verdict=accepted
else
    scan_verdict=refused
fi
check "an expansion that matched nothing is refused under must-find" "$scan_verdict" "refused"
check "and the unmatched pattern itself is not left in the list as a path" \
    "$(wc -l < "$scan_list" | tr -d ' ')" "0"

if scan_expand "$scan_list" may-be-empty 'the fixture' "$scan_dir"/nowhere/*.sh; then
    scan_verdict=accepted
else
    scan_verdict=refused
fi
check "the same empty expansion is accepted when the caller has said it may be" "$scan_verdict" "accepted"

# A misspelt policy must not read as the permissive one, which is the only way the permissive answer can
# be given by accident.
if scan_expand "$scan_list" maybe-empty 'the fixture' "$scan_dir"/nowhere/*.sh 2>/dev/null; then
    scan_verdict=accepted
else
    scan_verdict=refused
fi
check "an unknown empty-policy is refused rather than treated as permissive" "$scan_verdict" "refused"

# The far end of the filter: the list was read, and everything in it was skipped.
if scan_require_count 0 'the fixture' 2>/dev/null; then
    scan_verdict=accepted
else
    scan_verdict=refused
fi
check "a scan that read a full list and inspected none of it is refused" "$scan_verdict" "refused"

printf 'has HOME in it\n' > "$scan_dir/one.sh"
check "scan_grep passes a match through" \
    "$(scan_grep -c HOME "$scan_dir/one.sh")" "1"
check "and reports no match as an empty result, not as an error" \
    "$(scan_grep -c HOME "$scan_dir/two.sh" || echo ERROR)" "0"
# grep exits 2 here, and `|| true` — the idiom this replaces — turns that into the same empty string the
# line above produces. The two must not be the same answer.
if scan_grep -c HOME "$scan_dir/not-a-file.sh" >/dev/null 2>&1; then
    scan_verdict=accepted
else
    scan_verdict=refused
fi
check "but a path that does not exist is refused rather than reported as no match" \
    "$scan_verdict" "refused"
rm -rf "$scan_dir"
rm -f "$scan_list"

# --- no probe script names the HARNESS's home ------------------------------------------------------
#
# THE DEFECT, named. The container runs two unprivileged users, so `$HOME` inside a probe script is the
# HARNESS's home, /home/probe. The agent is launched with `HOME=$PROBE_AGENT_HOME`. A location argument
# written `"$HOME/.cline"` therefore watches a directory the agent never writes: the baseline is empty,
# the after-snapshot is empty, the delta is empty — and an empty delta is what this harness documents as
# the launch having changed nothing, unambiguously. The transcript then reads as clean isolation for an
# agent that may be leaking, which is the one outcome the whole harness exists to detect. All twelve
# scripts were written that way and nothing noticed, because `common.sh` states the contract in prose.
#
# THE WHOLE FILE IS SCANNED, not the call lines, and that is the point: four of the twelve (aider, amp,
# continue, opencode) compute the location into `default=` first and pass `"$default"`. A check that read
# only the arguments of probe_behaviour / probe_pristine would have been blind to the exact shape the
# repository already uses. probe_candidates takes no location argument at all today; a `$HOME` in a
# candidate's CONTENT would be the same mistake, and is covered by scanning everything.
#
# WHAT THIS CANNOT CATCH, written down because a green check that reads as more than it is caused the
# defect above, and because the enumeration check earlier in this file had to say the same:
#   * an indirect expansion — `v=HOME; eval "d=\$$v"`, or a location arriving through a variable set
#     somewhere this file never reads. The text `$HOME` never appears, so a grep cannot see it.
#   * a tilde. `~/.cline` expands to the HARNESS's home exactly as `$HOME/.cline` does, and is NOT
#     matched, deliberately: two probe scripts quote `~/...` paths out of VENDOR documentation, where the
#     tilde is the reader's home and correct. A check that flagged those would have to be argued away
#     twelve times, which is how a check stops being read.
#   * a hard-coded `/home/probe/...`, or any absolute path. Nothing here knows which paths are whose.
#   * a WRONG `$PROBE_AGENT_HOME` path. This checks the VARIABLE, not the location:
#     `$PROBE_AGENT_HOME/.clyne` passes, and only a container run would say otherwise.
#   * anything outside `sandbox/probes/*.sh` — a location named by `run.sh`, or passed in through the
#     environment.

# harness_home <file>: the lines on which a probe script names the harness's home. Empty is the pass.
#
# A FUNCTION, so the REJECTING half can be tested. A collector that fired for nothing would print `ok`
# twelve times and mean nothing — the failure mode two checks in this repository were measured to have.
#
# `scan_grep`, not `grep ... || true`. THIS IS WHERE THE THIRD INSTANCE OF THE CLASS LIVED. `|| true` is
# needed for grep's "no match" — which is the PASS here — but it also swallowed grep's status 2, and the
# loop below used to hand this function an unmatched glob's literal text, which names no file. A path that
# does not exist and a file with no `$HOME` in it produced the same empty string, and the empty string is
# what this function calls clean. scan_grep returns the hard error instead; the caller reports it.
harness_home() {
    scan_grep -nE '\$\{?HOME([^A-Za-z0-9_]|$)' "$1"
}

# THE MUTANT IS THE DEFECT, not merely something that reddens the check: the literal line `cline.sh`
# carried, at the argument position that decides what gets watched.
mutant=$(mktemp)
printf '%s\n' 'probe_behaviour cline "$HOME/.cline" env:CLINE_DATA_DIR @none' > "$mutant"
check "the check catches the defect it is named for" \
    "$(harness_home "$mutant")" '1:probe_behaviour cline "$HOME/.cline" env:CLINE_DATA_DIR @none'
# The other half of the same line, which must pass — otherwise the check is satisfied by any edit at all.
printf '%s\n' 'probe_behaviour cline "$PROBE_AGENT_HOME/.cline" env:CLINE_DATA_DIR @none' > "$mutant"
check "and passes once the same line names the agent's home" "$(harness_home "$mutant")" ""
# And the false positive that would make it unusable: four probe scripts name variables ENDING in HOME.
printf '%s\n' 'probe_strings cursor-agent CURSOR_CONFIG_DIR $XDG_CONFIG_HOME $PROBE_AGENT_HOME' > "$mutant"
check "and does not fire on a variable that merely ends in HOME" "$(harness_home "$mutant")" ""
# And the mutant that models the CLASS rather than the defect: the file this scan is pointed at is not
# there, while the thing being scanned for is still in the tree. `grep ... || true` returned the empty
# string here — indistinguishable from the passing line above it — and the loop below printed `ok`.
rm -f "$mutant"
if harness_home "$mutant" >/dev/null 2>&1; then
    scan_of_nothing=accepted
else
    scan_of_nothing=refused
fi
check "scanning a file that is not there is refused, not read as a clean file" \
    "$scan_of_nothing" "refused"

# The twelve probe scripts, collected ONCE, through the guard. Two loops scan this list; both used to
# iterate `"$root"/sandbox/probes/*.sh` directly, and POSIX sh has no `nullglob`.
probe_scripts=$(mktemp)
if scan_expand "$probe_scripts" must-find 'sandbox/probes/*.sh' "$root"/sandbox/probes/*.sh; then
    echo "ok   - the probe-script scan found files to read"
else
    echo "FAIL - the probe-script scan found no files under sandbox/probes/"
    failures=$((failures + 1))
fi

home_scanned=0
while IFS= read -r script; do
    name=$(basename "$script" .sh)
    case "$name" in
        # common.sh DEFINES the contract and quotes `$HOME` in explaining it; text.sh names no location.
        common | text) continue ;;
    esac
    home_scanned=$((home_scanned + 1))

    if ! named=$(harness_home "$script"); then
        echo "FAIL - $name.sh could not be scanned for the harness's home"
        failures=$((failures + 1))
        continue
    fi
    if [ -n "$named" ]; then
        echo "FAIL - $name.sh names the harness's home, not the agent's:"
        printf '%s\n' "$named" | sed 's/^/       /'
        failures=$((failures + 1))
        continue
    fi
    echo "ok   - $name.sh names the agent's home, not the harness's"
done < "$probe_scripts"

# A NON-EMPTY LIST IS NOT THE SAME QUESTION. The glob matches common.sh and text.sh, which the loop skips
# by name, so moving the twelve agent scripts aside leaves scan_expand satisfied and this loop reporting
# nothing at all — zero `ok` lines, zero failures, exit 0. The count is what was actually read.
if scan_require_count "$home_scanned" 'probe scripts scanned for the harness home'; then
    echo "ok   - the harness's-home scan read at least one probe script"
else
    echo "FAIL - the harness's-home scan read no probe script"
    failures=$((failures + 1))
fi

# --- the step order, over every probe script -------------------------------------------------------

# §7.3: "Every probe script then follows this order exactly. The order is part of the contract, because
# probe_behaviour's delta is only attributable if nothing has run the agent before it." A contract that
# only exists in prose is one a twelfth probe script can quietly break — and it would not fail, it would
# produce a plausible, wrong measurement: a baseline taken after a launch attributes the launch's own
# files to the install.
# order_fault <order-string>: what the contract says is wrong with this step order, or nothing.
#
# A FUNCTION, rather than a case statement inline in the loop below, so that the REJECTING half can be
# tested. The loop only ever feeds it orders that should pass; a check that proves acceptance and never
# rejection is what let the candidates-before-behaviour inversion through for three scripts while printing
# `ok` for each of them.
order_fault() {
    case "$1" in
        # install, version, help, strings, then one or more behaviour/candidates steps.
        "npm version help strings "* | "uv version help strings "* | "script version help strings "*) ;;
        *) echo "does not follow the six-step order"; return 0 ;;
    esac
    case "$1" in
        *behaviour*) ;;
        *) echo "never measures behaviour"; return 0 ;;
    esac
    # The rationale above, made mechanical. `common.sh` learns a default location's pre-launch state the
    # first time that location is named, and only `probe_behaviour` names one — so a candidate sweep that
    # runs first launches the agent before anything has archived the location it writes to, and the
    # behaviour delta that follows cancels to nothing. Measured with a stub that recreates its config on
    # every launch: candidates-then-behaviour gave a 0-byte delta where behaviour-first named the file.
    case "$1" in
        *candidates*behaviour*) echo "sweeps candidates before it measures behaviour"; return 0 ;;
    esac
    return 0
}

# The distractor: the check must REJECT the inversion, not merely accept the right order.
check "the order check rejects a sweep before the behaviour step" \
    "$(order_fault "npm version help strings candidates behaviour ")" \
    "sweeps candidates before it measures behaviour"
check "the order check accepts the behaviour step before the sweep" \
    "$(order_fault "uv version help strings behaviour candidates ")" ""

order_scanned=0
while IFS= read -r script; do
    name=$(basename "$script" .sh)
    case "$name" in
        common | text) continue ;;
    esac
    order_scanned=$((order_scanned + 1))

    # The step each call appears at, so the ORDER can be asserted rather than mere presence.
    #
    # grep is the LEFT-HAND SIDE of this pipe, so its status is discarded and cannot be recovered — POSIX
    # sh has no `pipefail`. That is safe HERE and nowhere near generally: an unreadable script yields an
    # empty order string, and the empty string matches none of order_fault's accepting arms, so it comes
    # back "does not follow the six-step order" and reddens. This loop fails CLOSED on the same input the
    # harness-home loop above used to pass on, which is why the two sat side by side disagreeing.
    order=$(grep -n '^probe_\(npm_install\|uv_install\|script_install\|version\|help\|strings\|behaviour\|candidates\)' \
        "$script" | sed 's/:.*probe_/ /' | sed 's/_install//' | awk '{print $2}' | tr '\n' ' ')

    fault=$(order_fault "$order")
    if [ -n "$fault" ]; then
        echo "FAIL - $name.sh $fault: '$order'"
        failures=$((failures + 1))
        continue
    fi
    echo "ok   - $name.sh follows the six-step order"
done < "$probe_scripts"

if scan_require_count "$order_scanned" 'probe scripts scanned for the six-step order'; then
    echo "ok   - the step-order scan read at least one probe script"
else
    echo "FAIL - the step-order scan read no probe script"
    failures=$((failures + 1))
fi
rm -f "$probe_scripts"

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all probe-harness checks passed"
