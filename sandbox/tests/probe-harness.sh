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

check() {
    if [ "$2" = "$3" ]; then
        echo "ok   - $1"
    else
        echo "FAIL - $1: expected '$3', got '$2'"
        failures=$((failures + 1))
    fi
}

# A probe script, written to a temporary directory and run exactly as the container runs one: from the
# repository root, because that is where `sandbox/run.sh:164` leaves a probe and it is how common.sh
# resolves its own sibling files.
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
. sandbox/probes/common.sh
$1
SCRIPT
    mkdir -p "$out/bin"
    set +e
    sh "$script" > "$out/stdout" 2>&1
    probe_status=$?
    set -e
    # What `sandbox/run.sh:198` does once the container has stopped: lift `failures` and `steps` out of
    # the state directory — which is NOT under the writable /out mount, so the measured party cannot
    # forge them — into the results directory, which is where `transcript.sh:85` reads `steps` from.
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

# The measured party can WRITE /out: it is mounted read-write (`run.sh:183,188`, beside `/src:ro`) and the
# agent — or the install scripts `npm install --global` runs before the agent exists — shares that
# directory at the same uid. When `failures` lived there, truncating it made `probe_finish` exit 0 and the
# matrix job go green, and §5.3's byte-comparison could not tell: artifact and committed file both derive
# from the forged bytes. This stub does exactly that, and the run must still be recorded as failed.
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

# --- the privilege boundary, COMPILED OUT ----------------------------------------------------------
#
# READ THIS BEFORE TRUSTING THE GREEN. The container runs the harness as `probe` and everything measured
# as `agent`, and that split is the only thing that makes the bookkeeping above tamper-proof rather than
# merely moved. THIS SUITE DOES NOT EXERCISE IT. It runs on the maintainer's host, where there is no
# `agent` user and no `sudo`, so `common.sh` detects their absence and every crossing degrades to a
# direct call — deliberately, because otherwise the suite could not run this file at all. What follows
# therefore tests the DEGRADATION and the contract around it, never the boundary: that a step still runs
# and is still recorded with the switch compiled out, that the privilege plumbing stays out of the
# evidence, and that the flag which selects the uid cannot leak into the next step. The boundary itself
# is verified only by a real container run. A guarantee no test covers must not read as though one does.

run_probe 'probe_record_agent direct true'
check "a step that would cross the boundary still runs when it is compiled out" "$probe_status" "0"
check "and is recorded exactly as a harness step is" \
    "$(cat "$PROBE_OUT_DIR/direct.exit-code")" "0"

# The transcript states what the VENDOR documents. `sudo -n -u agent -- env HOME=... npm install ...`
# states that plus a fact about this harness, and only the first is evidence about the agent.
run_probe 'probe_record_agent boom sh -c "exit 3"'
check "the privilege switch is not written into the recorded command" \
    "$(cat "$PROBE_OUT_DIR/boom.cmd")" "sh -c exit 3"
check "a failed agent-side step still fails the probe" "$probe_status" "1"

# The flag decides WHICH UID a command runs at, so a value left set after a call would silently put the
# next step — a snapshot, a restore, anything the harness does for itself — on the wrong side.
run_probe 'probe_record_agent one true
printf "[%s]\n" "$PROBE_AS_AGENT" > "$PROBE_OUT/flag"'
check "the agent flag does not leak past the call that set it" \
    "$(cat "$PROBE_OUT_DIR/flag")" "[]"

# The twelve probe scripts name the agent's default locations through this, because under two users
# `$HOME` is the HARNESS's home and not where the agent writes.
run_probe 'printf "%s\n" "$PROBE_AGENT_HOME" > "$PROBE_OUT/agent-home"'
check "PROBE_AGENT_HOME names the agent's home, not the harness's" \
    "$(cat "$PROBE_OUT_DIR/agent-home")" "/home/agent"

# --- probe_behaviour ------------------------------------------------------------------------------

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
# missing, empty and comment-only each exit 2 while `{}` is accepted (aider.rs:17-18).
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

# A default location is not always a directory — Aider's is the file `.aider.conf.yml` (aider.rs:15) — and
# it is not always the agent CREATING something: this stub DELETES its own config, which is why the
# restore is an exact unpack of the pre-launch state rather than "remove whatever appeared". A heuristic
# that only undoes additions leaves the second launch with nothing left to delete, and the `- f` row that
# says the agent removed its config never appears again.
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
check "the refusal is recorded as a failed install" \
    "$(cat "$PROBE_OUT_DIR/install.exit-code")" "2"

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

for script in "$root"/sandbox/probes/*.sh; do
    name=$(basename "$script" .sh)
    case "$name" in
        common | text) continue ;;
    esac

    # The step each call appears at, so the ORDER can be asserted rather than mere presence.
    order=$(grep -n '^probe_\(npm_install\|uv_install\|script_install\|version\|help\|strings\|behaviour\|candidates\)' \
        "$script" | sed 's/:.*probe_/ /' | sed 's/_install//' | awk '{print $2}' | tr '\n' ' ')

    fault=$(order_fault "$order")
    if [ -n "$fault" ]; then
        echo "FAIL - $name.sh $fault: '$order'"
        failures=$((failures + 1))
        continue
    fi
    echo "ok   - $name.sh follows the six-step order"
done

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all probe-harness checks passed"
