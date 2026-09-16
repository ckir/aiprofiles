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
# repository root, because that is where `sandbox/run.sh:135` leaves a probe and it is how common.sh
# resolves its own sibling files.
run_probe() {
    out=$(mktemp -d)
    script=$(mktemp)
    cat > "$script" <<SCRIPT
cd $root
PROBE_OUT=$out
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

run_probe 'probe_record slow sleep 30'
check "a hanging step is killed and recorded" "$(cat "$PROBE_OUT_DIR/slow.exit-code")" "124"
check "a hanging step fails the probe" "$probe_status" "1"

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
for script in "$root"/sandbox/probes/*.sh; do
    name=$(basename "$script" .sh)
    case "$name" in
        common | text) continue ;;
    esac

    # The step each call appears at, so the ORDER can be asserted rather than mere presence.
    order=$(grep -n '^probe_\(npm_install\|uv_install\|script_install\|version\|help\|strings\|behaviour\|candidates\)' \
        "$script" | sed 's/:.*probe_/ /' | sed 's/_install//' | awk '{print $2}' | tr '\n' ' ')

    case "$order" in
        # install, version, help, strings, then one or more behaviour/candidates steps.
        "npm version help strings "* | "uv version help strings "* | "script version help strings "*) ;;
        *)
            echo "FAIL - $name.sh does not follow the six-step order: '$order'"
            failures=$((failures + 1))
            continue
            ;;
    esac
    case "$order" in
        *behaviour*) ;;
        *)
            echo "FAIL - $name.sh never measures behaviour"
            failures=$((failures + 1))
            continue
            ;;
    esac
    echo "ok   - $name.sh follows the six-step order"
done

if [ "$failures" -ne 0 ]; then
    echo "$failures check(s) failed"
    exit 1
fi
echo "all probe-harness checks passed"
