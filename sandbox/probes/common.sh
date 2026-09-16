# Shared steps for probe scripts. Sourced by sandbox/probes/<agent>.sh inside the sandbox container, from a
# copy of the checkout; results go to /out, which sandbox/run.sh copies back to target/sandbox/.
#
# A probe installs one agent, records its version and help text, and measures whether the agent's own
# isolation mechanism actually moves its files (SP4 design §7.3). It never launches the agent through
# `agent-profile`: the CLI refuses an agent word it does not know, and the nine SP4 agents are unknown to
# the binary at the commit that runs the probe.

set -eu

# Sourced by relative path, because the container runs a probe from the copied checkout's root
# (`sandbox/run.sh:148` cds to /home/probe/work) and so does the shell suite. A missing file here would
# otherwise surface as `probe_strip_ansi: not found` three steps later, naming a symptom instead of a cause.
if [ ! -f sandbox/probes/text.sh ]; then
    echo "probe: run from the repository root; sandbox/probes/text.sh is not here" >&2
    exit 2
fi
. sandbox/probes/text.sh

# Where the probe puts the profile it points the agent at. Fixed, so a probe script never invents one.
# `/out` in the container; overridable so the shell suite can run this file outside one.
PROBE_OUT=${PROBE_OUT:-/out}
# Where the harness keeps the two files that decide the RUN'S STATUS: `failures` and `steps`.
#
# NOT under $PROBE_OUT. `/out` is a read-write host mount (`run.sh:167,172`, beside `/src:ro`), and the
# measured party runs at the same uid inside the same container — an agent, or the install scripts an
# `npm install --global` runs before the agent even exists. `probe_finish` derives the run's exit status
# from `failures`, that status becomes the container's, the matrix job's, and the conclusion
# `verify-transcripts.sh` requires before it accepts a committed transcript as evidence. Truncating
# `failures` under /out therefore turned a failed probe green, and §5.3's byte-comparison could not see
# it: the uploaded artifact and the committed file both derive from the forged bytes, so they match.
#
# This protects the BOOKKEEPING only. `help.txt`, `strings.txt` and every other recorded artefact are
# agent-produced and always were; what is restored is that the agent cannot control WHETHER its run is
# recorded as failed. `run.sh:182` copies this directory into the results directory after the container
# exits, so `transcript.sh:85` still finds `steps` where it has always been.
PROBE_STATE=${PROBE_STATE:-/home/probe/.probe-state}
PROBE_TARGET=${PROBE_TARGET:-/home/probe/probe-target}
# How long any one recorded command may run. A probe never waits for input; a hang is a recorded fact, not
# a job that burns its whole budget.
PROBE_TIMEOUT=${PROBE_TIMEOUT:-300}
# The version the maintainer asked for, or empty for "whatever the registry serves" (§7.4). `run.sh:158`
# passes it as the probe script's first argument — charset-checked at `run.sh:55` and quoted there, so it
# reaches this file as one word — and a sourced file sees its caller's positional parameters, so no probe
# script has to thread it through.
PROBE_VERSION=${PROBE_VERSION:-${1:-}}
# The candidate file's name inside the profile directory. Some agents key on the extension — Continue
# reads a `config.yaml` and Amp a `settings.json` — so a probe that must name it can, and one whose agent
# does not care leaves it alone.
PROBE_CONFIG_NAME=${PROBE_CONFIG_NAME:-config}

mkdir -p "$PROBE_STATE"
: > "$PROBE_STATE/failures"
: > "$PROBE_STATE/steps"
mkdir -p "$PROBE_TARGET"
# Where `probe_pristine` keeps the pre-launch copy of each watched default location — beside `failures`
# and `steps`, and for the same reason: the measured party must not be able to rewrite the state the
# harness restores it to, and /out is writable by it. Cleared rather than merely created, because the
# archives are numbered from the index and a stale `1.tar` beside a fresh index would be restored over a
# location it was never taken from.
rm -rf "$PROBE_STATE/pristine"
mkdir -p "$PROBE_STATE/pristine"
: > "$PROBE_STATE/pristine/index"

# probe_record <name> <command...>: run a command under a timeout, keep its output and exit code in /out.
#
# It returns 0 even when the command failed, and accumulates the failure instead. That is deliberate: these
# scripts run under `set -e`, so returning the command's status would abort the probe at its first failure
# and leave every later step unrecorded — and the later steps are the evidence. `probe_finish`, wired to an
# EXIT trap so a script cannot forget it, is what turns an accumulated failure into a non-zero exit.
probe_record() {
    name=$1
    shift
    # What was run, beside what it printed. The transcript has to say `install: npm install --global
    # @google/gemini-cli@0.60.0` rather than just the install's output, and only the probe knows the
    # command after PROBE_VERSION has been folded into it.
    printf '%s\n' "$*" > "$PROBE_OUT/$name.cmd"
    set +e
    timeout --kill-after=10s "$PROBE_TIMEOUT" "$@" > "$PROBE_OUT/$name.txt" 2>&1
    status=$?
    set -e
    echo "$status" > "$PROBE_OUT/$name.exit-code"
    # Appended in STEP ORDER, which a directory listing cannot reconstruct: sorted by name, `behaviour`
    # precedes `install`, and a transcript in that order reads as an agent launched before it existed.
    printf '%s %s\n' "$name" "$status" >> "$PROBE_STATE/steps"
    if [ "$status" -ne 0 ]; then
        echo "$name $status" >> "$PROBE_STATE/failures"
    fi
    return 0
}

# probe_fail <name> <status>: record a failed step that was not a single recorded command.
#
# It exists so that a step which fails for its own reasons — an executable that vanished after a
# successful install, an installer asked for a version it cannot honour — lands in all three of the same
# places a failed command does. A step that reported only to stderr would leave `exit-codes:` in the
# transcript claiming the probe ran clean.
#
# It deliberately does NOT share a helper with `probe_record`'s failure path. It writes the same three
# records, but it is the whole of its caller's bookkeeping, whereas `probe_record` has already written
# two of them by the time it knows the status. Routing one through the other appended the step to
# `steps` twice, which reads as the probe having run it twice.
probe_fail() {
    echo "$2" > "$PROBE_OUT/$1.exit-code"
    echo "$1 $2" >> "$PROBE_STATE/failures"
    printf '%s %s\n' "$1" "$2" >> "$PROBE_STATE/steps"
}

# probe_finish: write the summary and exit non-zero if any recorded command failed.
probe_finish() {
    status=$?
    if [ -s "$PROBE_STATE/failures" ]; then
        echo "probe: recorded failures:" >&2
        cat "$PROBE_STATE/failures" >&2
        # A script that already failed for its own reason KEEPS that status. `exit 2` means "you asked
        # for something this probe cannot do" and 1 means "the probe ran and a step failed" — a maintainer
        # re-runs after the first and investigates after the second. Flattening both to 1 would hide the
        # distinction behind a `failures` file that looks the same either way.
        [ "$status" -ne 0 ] || status=1
    fi
    exit "$status"
}
trap probe_finish EXIT

# --- Step 1: install ------------------------------------------------------------------------------
#
# Each installer is a separate function rather than one with a mode argument, because only some of them
# can honour a pinned version and the difference has to be visible at the call site. All three record
# under the name `install`, so the transcript's step names are the same six for every agent.

# probe_npm_install <package>: install one npm package globally, at the requested version if given.
probe_npm_install() {
    probe_pkg=$1
    shift
    # Extra flags come from the vendor's own documented command, not from us: Pi documents
    # `npm install -g --ignore-scripts @earendil-works/pi-coding-agent`, and dropping the flag would
    # measure an install the vendor does not describe.
    probe_record install npm install --global "$@" "$probe_pkg${PROBE_VERSION:+@$PROBE_VERSION}"
}

# probe_uv_install <package> <python-version>: install one Python tool, at the requested version if given.
# The interpreter is pinned by the caller; CONTRIBUTING.md:108-109's pinning guidance covers exactly this.
probe_uv_install() {
    probe_record install uv tool install --python "$2" "$1${PROBE_VERSION:+==$PROBE_VERSION}"
}

# probe_script_install <url>: run a vendor install script, as the vendor documents it.
#
# A version request is REFUSED rather than ignored. `run.sh` accepts one for every agent, but a vendor
# script takes whatever it takes; silently dropping the argument would produce a transcript whose
# `install:` line names a version the installer never saw, and §5.3 then commits that false statement
# byte-for-byte. Failing is recoverable — re-run without the argument; a wrong transcript is not.
probe_script_install() {
    if [ -n "$PROBE_VERSION" ]; then
        echo "probe: $1 takes no version argument (asked for $PROBE_VERSION)" >&2
        probe_fail install 2
        exit 2
    fi
    # The interpreter is the vendor's, because the script is the vendor's. Both script-installed agents
    # document `| bash`, and a bash script run under dash fails in ways that would be recorded as the
    # agent failing to install.
    probe_record install sh -c "curl -fsSL '$1' | ${2:-sh}"
}

# --- Step 2: version ------------------------------------------------------------------------------

# probe_version <executable>: record `--version` verbatim, and beside it the one token SP4b types into
# the registry.
#
# The extraction rule is §7.3 step 2's: the first `[A-Za-z0-9._-]` run in the output that contains a
# digit. It lives here rather than in a reviewer's head because Gate A resolves a path from the recorded
# version and SP4b resolves the same path from the registry; if the two derive the token differently, the
# gate fails on a file that exists. `aider --version` prints `aider 0.86.2` — a line, not a token, and the
# space alone violates Gate A's charset.
#
# The raw capture stays verbatim: it is the evidence. The extraction reads a stripped copy, because a
# coloured banner begins `ESC[0m`, whose `0m` is a digit-bearing run that Gate A would happily accept.
probe_version() {
    probe_record version "$1" --version
    probe_extracted=$(
        probe_strip_ansi < "$PROBE_OUT/version.txt" \
            | tr -cs 'A-Za-z0-9._-' '\n' \
            | grep -m1 '[0-9]' \
            || true
    )
    # No such run means the agent printed no version: a §9 outcome-2 probe. `unknown` is the encoding
    # Gate C keys on to force the adapter to Experimental, so the probe states it rather than leaving the
    # field empty for a human to fill in.
    printf '%s\n' "${probe_extracted:-unknown}" > "$PROBE_OUT/version.extracted"
}

# --- Step 3: help ---------------------------------------------------------------------------------

# probe_help <executable>: record `--help`, the artefact Gate A reads the mechanism token from.
probe_help() {
    probe_record help "$1" --help
}

# --- Step 4: strings ------------------------------------------------------------------------------

# probe_strings <executable> [documented-variable...]
#
# Two questions, one file: does each DOCUMENTED credential variable actually appear in what was installed,
# and what UNDOCUMENTED ones appear beside them? The second is the one that pays. Claude's adapter cites
# `CLAUDE_CODE_USE_BEDROCK` and three OAuth variables as "binary strings measured" (`claude.rs:27-29`) —
# no page documented them, and each one is a way a user defeats the isolation the adapter promises.
#
# There is no `strings(1)` in the image: `sandbox/Containerfile:17` installs no binutils. `grep -a` reads
# a binary as text and is already a dependency, so this uses that rather than growing the image.
probe_strings() {
    probe_exe=$1
    shift
    probe_out_file=$PROBE_OUT/strings.txt
    probe_resolved=$(command -v "$probe_exe" 2>/dev/null || true)
    if [ -z "$probe_resolved" ]; then
        printf '# %s is not on PATH; nothing to scan\n' "$probe_exe" > "$probe_out_file"
        probe_fail strings 127
        return 0
    fi
    probe_real=$(readlink -f "$probe_resolved")
    probe_dir=$(dirname "$probe_real")
    printf '# %s -> %s\n' "$probe_resolved" "$probe_real" > "$probe_out_file"

    # The resolved file and its siblings. An npm package's bundle sits beside its entry point and a
    # script-installed agent is one binary, so one directory covers both shapes. `-size -64M` keeps a
    # vendored toolchain from turning this step into the job's time limit.
    for probe_var in "$@"; do
        # Tested by what grep PRINTS, not by find's exit status. `-exec ... +` batches, and it reports
        # failure when ANY batch's grep found nothing — so a variable present in the first of two batches
        # would be recorded ABSENT, which is the answer that ends an investigation early.
        if find "$probe_dir" -maxdepth 1 -type f -size -64M \
            -exec grep -aFl -e "$probe_var" {} + 2>/dev/null | grep -q .; then
            printf 'documented %s: present\n' "$probe_var" >> "$probe_out_file"
        else
            printf 'documented %s: ABSENT\n' "$probe_var" >> "$probe_out_file"
        fi
    done

    printf '#\n# variable-shaped tokens that name a credential or a location, documented or not\n' \
        >> "$probe_out_file"
    find "$probe_dir" -maxdepth 1 -type f -size -64M \
        -exec grep -aohE '[A-Z][A-Z0-9_]{3,}' {} + 2>/dev/null \
        | grep -E '(KEY|TOKEN|SECRET|CREDENTIAL|PASSWORD|AUTH|HOME|CONFIG|DATA_DIR|PROFILE|SETTINGS)' \
        | LC_ALL=C sort -u \
        | probe_excerpt 200 >> "$probe_out_file"
}

# --- Steps 5 and 6: baseline and behaviour --------------------------------------------------------

# probe_snapshot <name> <dir...>: record a sorted listing of each directory, for a before/after comparison.
probe_snapshot() {
    name=$1
    shift
    : > "$PROBE_OUT/$name.txt"
    for dir in "$@"; do
        echo "# $dir" >> "$PROBE_OUT/$name.txt"
        if [ -d "$dir" ]; then
            find "$dir" -printf '%y %p\n' 2>/dev/null | LC_ALL=C sort >> "$PROBE_OUT/$name.txt"
        elif [ -e "$dir" ]; then
            # A default location is not always a directory: Aider's is the file `.aider.conf.yml`
            # (`aider.rs:15`). Reporting an existing file as `(absent)` would read as the agent having
            # written nothing to its default location, which is the finding the whole probe is for.
            find "$dir" -maxdepth 0 -printf '%y %p\n' 2>/dev/null >> "$PROBE_OUT/$name.txt"
        else
            echo "(absent)" >> "$PROBE_OUT/$name.txt"
        fi
    done
}

# probe_delta <baseline-file> <after-file> <out-file>: what the launch actually changed.
#
# This is a COMPUTED difference, not the after-state under a suggestive name. The distinction is the whole
# measurement: the probe creates the target directory itself, and for a file mechanism it writes the
# candidate file, so a post-launch listing contains the probe's own bytes. Reading that listing as "what
# the agent wrote" yields a false `ConfigIsolation: Supported` carrying a `measured:` basis — and every
# gate still passes, because the gates check a claim's shape, not whether the measurement behind it meant
# anything.
#
# `+` is a path that appeared, `-` one that went away. An empty file means the launch changed nothing,
# unambiguously, which is a finding rather than a gap.
probe_delta() {
    LC_ALL=C sort "$1" > "$PROBE_OUT/.delta-before"
    LC_ALL=C sort "$2" > "$PROBE_OUT/.delta-after"
    {
        comm -13 "$PROBE_OUT/.delta-before" "$PROBE_OUT/.delta-after" | sed 's/^/+ /'
        comm -23 "$PROBE_OUT/.delta-before" "$PROBE_OUT/.delta-after" | sed 's/^/- /'
    } > "$3"
    rm -f "$PROBE_OUT/.delta-before" "$PROBE_OUT/.delta-after"
}

# probe_label <mechanism>: the filename-safe form a mechanism's artefacts are named by.
probe_label() {
    printf '%s' "$1" | tr -c 'A-Za-z0-9_' '-'
}

# probe_prepare_target [candidate]: an empty profile directory, holding the candidate file if one is given.
#
# `@none` means "create no file", which is a content worth testing in its own right: Aider refuses a
# MISSING `.aider.conf.yml` exactly as it refuses an empty one (`aider.rs:17-18`), and an adapter that
# creates nothing would hit that.
probe_prepare_target() {
    rm -rf "$PROBE_TARGET"
    mkdir -p "$PROBE_TARGET"
    case "${1:-@none}" in
        @none) ;;
        *) printf '%s' "$1" > "$PROBE_TARGET/$PROBE_CONFIG_NAME" ;;
    esac
}

# --- keeping every launch attributable, not just the first ----------------------------------------
#
# `probe_prepare_target` resets the TARGET before each launch; nothing reset the DEFAULT LOCATION, and an
# agent initialises that location on its first launch. So a probe that launches more than once — cline's
# three mechanisms, opencode's two, every candidate in an acceptance sweep — took its second baseline over
# a location the first launch had already populated, and the files whose appearance IS the
# isolation-failure signal sat on both sides of `comm` and cancelled out of the delta. Measured with a stub
# that ignores the mechanism and recreates its config on every launch: the delta was 0 bytes where the same
# stub measured first produced `+ f .aider.chat.history.md` and `+ f .aider.conf.yml`. A reader does not see
# a gap — it reads as the mechanism isolating, which is the exact wrong answer.
#
# Restoration is EXACT: a tar of the pre-launch state, unpacked over a cleared location. Deleting whatever
# appeared would be cheaper and wrong — an agent that rewrites or deletes its own config between launches
# leaves a MODIFIED file that a delete-what-is-new pass keeps, and the next delta is then missing the row
# it exists to record.
#
# HOW THE HELPERS LEARN THE LOCATIONS. `probe_behaviour` is told its default location as an argument;
# `probe_candidates` is not told at all, because §8.4's question is about acceptance rather than about
# where files land. The choice made here is LAZY ON FIRST USE — `probe_behaviour` registers the location
# it is handed the first time it sees it, and every later launch of either kind restores everything
# registered so far. The alternative, a `probe_pristine` line or a variable in each of the twelve probe
# scripts, was rejected because it puts the contract in twelve places that can each forget it; lazy
# registration keeps it in this file, and the nine probe scripts that launch only through `probe_behaviour`
# need no change at all.
#
# Lazy registration has one requirement, and it is the one §7.3 already states: the FIRST launch of a probe
# must be a `probe_behaviour`, or the archive captures a default location an earlier sweep has already
# dirtied. `sandbox/tests/probe-harness.sh`'s order check now REJECTS a script that sweeps candidates
# first, so that is enforced mechanically rather than assumed — and a probe whose first launch cannot be a
# behaviour step can call `probe_pristine` itself, up front, which is what the three sweep scripts do.

# probe_pristine_id <location>: the number this location was registered under, or nothing if it was not.
probe_pristine_id() {
    while read -r probe_id_n probe_id_p; do
        if [ "$probe_id_p" = "$1" ]; then
            printf '%s' "$probe_id_n"
            return 0
        fi
    done < "$PROBE_STATE/pristine/index"
    return 0
}

# probe_pristine <location...>: record each location's pre-launch state, once, the first time it is seen.
#
# Idempotent by design: a location already registered KEEPS its first archive, so calling this again after
# a launch cannot quietly re-baseline the measurement onto a dirtied location.
probe_pristine() {
    for probe_loc in "$@"; do
        [ -n "$probe_loc" ] || continue
        [ -z "$(probe_pristine_id "$probe_loc")" ] || continue
        probe_pn=$(($(wc -l < "$PROBE_STATE/pristine/index") + 1))
        printf '%s %s\n' "$probe_pn" "$probe_loc" >> "$PROBE_STATE/pristine/index"
        if [ ! -e "$probe_loc" ]; then
            # A location the agent has not created yet. Its ABSENCE is the state to restore, and a marker
            # records it: "registered and absent" and "never registered" must not look the same to
            # `probe_restore_default`, or a failed archive would read as absence and delete a real one.
            : > "$PROBE_STATE/pristine/$probe_pn.absent"
        elif ! tar -cf "$PROBE_STATE/pristine/$probe_pn.tar" \
            -C "$(dirname "$probe_loc")" "$(basename "$probe_loc")" 2>/dev/null; then
            rm -f "$PROBE_STATE/pristine/$probe_pn.tar"
            echo "probe: could not archive $probe_loc before the first launch" >&2
            probe_fail "pristine-$probe_pn" 1
        fi
    done
}

# probe_restore_default <location>: put the location back the way `probe_pristine` found it.
probe_restore_default() {
    probe_rloc=$1
    # A destructive step, so the two values that would make it catastrophic are refused outright rather
    # than trusted to never be passed.
    if [ -z "$probe_rloc" ] || [ "$probe_rloc" = / ]; then
        echo "probe: refusing to restore '$probe_rloc'" >&2
        return 0
    fi
    probe_rn=$(probe_pristine_id "$probe_rloc")
    # Not registered: there is no pre-launch state to restore TO, and deleting the location would destroy
    # the very evidence the caller is about to snapshot.
    [ -n "$probe_rn" ] || return 0
    if [ -f "$PROBE_STATE/pristine/$probe_rn.tar" ]; then
        rm -rf "$probe_rloc"
        mkdir -p "$(dirname "$probe_rloc")"
        # stdin is closed to it because callers iterate the index over the loop's stdin; `-f` means tar
        # never wants stdin anyway, and this makes that independent of tar's implementation.
        if ! tar -xf "$PROBE_STATE/pristine/$probe_rn.tar" \
            -C "$(dirname "$probe_rloc")" < /dev/null; then
            echo "probe: could not restore $probe_rloc from its pristine copy" >&2
            probe_fail "restore-$probe_rn" 1
        fi
    elif [ -f "$PROBE_STATE/pristine/$probe_rn.absent" ]; then
        rm -rf "$probe_rloc"
    else
        echo "probe: no pristine copy of $probe_rloc; leaving it as it is" >&2
    fi
}

# probe_restore_known: restore every location registered so far.
#
# `probe_candidates` has no location argument, so this is how a sweep runs each candidate against the same
# default location the one before it saw. That also makes the sweep's own exit codes attributable: state a
# previous candidate left behind can decide whether the next one is accepted.
probe_restore_known() {
    while read -r probe_kn probe_kp; do
        [ -n "$probe_kn" ] || continue
        probe_restore_default "$probe_kp"
    done < "$PROBE_STATE/pristine/index"
}

# probe_apply <name> <executable> <mechanism> <arg>...: run the agent once with the mechanism applied.
#
# The mechanism vocabulary is FOUR words, not the two §7.3 first named, because two cannot express what §8
# asks the probe to test. `OPENCODE_CONFIG` is a variable naming a FILE (§8.2) and Cline's `--data-dir` is
# a flag naming a DIRECTORY (§8.3). Under an `env:`/`flag:` vocabulary the probe would have pointed a file
# variable at a directory and a directory flag at a file, the agent would have refused, and the refusal
# would have been recorded as evidence that the mechanism does not isolate — a wrong answer that looks
# like a measurement.
#
#   env:<VAR>        the variable is set to the profile directory
#   envfile:<VAR>    the variable is set to a file inside it
#   flagdir:<FLAG>   the flag is passed the profile directory
#   flagfile:<FLAG>  the flag is passed a file inside it
probe_apply() {
    probe_name=$1
    probe_exe=$2
    probe_mech=$3
    shift 3
    case "$probe_mech" in
        env:*)
            probe_record "$probe_name" env "${probe_mech#env:}=$PROBE_TARGET" "$probe_exe" "$@" ;;
        envfile:*)
            probe_record "$probe_name" env "${probe_mech#envfile:}=$PROBE_TARGET/$PROBE_CONFIG_NAME" "$probe_exe" "$@" ;;
        flagdir:*)
            probe_record "$probe_name" "$probe_exe" "${probe_mech#flagdir:}" "$PROBE_TARGET" "$@" ;;
        flagfile:*)
            probe_record "$probe_name" "$probe_exe" "${probe_mech#flagfile:}" "$PROBE_TARGET/$PROBE_CONFIG_NAME" "$@" ;;
        *)
            echo "probe: unknown mechanism $probe_mech" >&2
            probe_fail "$probe_name" 2
            return 1
            ;;
    esac
}

# probe_behaviour <executable> <default-location> <mechanism> <candidate|@none> [launch-arg...]
#
# Steps 5 and 6 as a pair: baseline both locations, apply the agent's own isolation mechanism, launch, and
# record what changed at both. Repeat per mechanism under test, each repetition re-baselining first — a
# second launch measured against the first launch's baseline is not attributable, which is the whole point
# of step 5.
#
# The first argument is the EXECUTABLE, not the agent id: they differ for kiro (kiro-cli), cursor
# (cursor-agent) and continue (cn), and this file has no registry to look one up in.
#
# The launch arguments default to `--version` and each probe script overrides them where it can, because
# `--version` is a FLOOR, not a good measurement: an agent that exits before initialising writes nothing,
# and an empty delta then reads as "the mechanism does not move anything" when it means "nothing was asked
# to move". Which cheap command makes a given agent initialise is per-agent and itself unmeasured, so a
# probe that has no better answer records the empty delta as the fact it is rather than inventing one.
probe_behaviour() {
    probe_exe=$1
    probe_default=$2
    probe_mech=$3
    probe_cand=${4:-@none}
    # `shift` is a special builtin: shifting past $# is an error that terminates a non-interactive POSIX
    # shell outright, so the count is bounded rather than guarded with `|| true`.
    if [ $# -ge 4 ]; then shift 4; else shift $#; fi
    [ $# -gt 0 ] || set -- --version
    probe_lbl=$(probe_label "$probe_mech")

    # Measurement n has to be attributable, not just measurement 1: register the default location's
    # pre-launch state the first time it is seen, and put it back before every launch after that. Without
    # this, cline's second and third mechanisms and opencode's second read as clean because the FIRST
    # launch already wrote the files — and those repetitions cannot be reordered away, because measuring
    # three mechanisms means launching three times.
    # shellcheck disable=SC2086 # <default-location> is a whitespace-separated LIST; the split is the point
    probe_pristine $probe_default
    # shellcheck disable=SC2086
    for probe_bloc in $probe_default; do
        probe_restore_default "$probe_bloc"
    done

    # Baseline BOTH locations, after the install and after anything this probe itself created. The install
    # ran vendor code, and a file mechanism means the probe wrote the candidate; neither is the agent's
    # doing, and attributing them to the agent would read as isolation that did not happen.
    probe_prepare_target "$probe_cand"
    # <default-location> is a whitespace-separated LIST, deliberately unquoted below. OpenCode needs two:
    # `OPENCODE_CONFIG_DIR` governs the config directory while `auth.json` lives under the XDG data
    # directory (§8.2), and watching only the first would record "credentials did not move" as an absence
    # of evidence rather than the measured fact §6 says is enough to claim NotSupported honestly.
    # shellcheck disable=SC2086 # the caller supplies literal paths, and the split is the point
    probe_snapshot "baseline-$probe_lbl" "$PROBE_TARGET" $probe_default
    probe_apply "behaviour-$probe_lbl" "$probe_exe" "$probe_mech" "$@"
    # shellcheck disable=SC2086
    probe_snapshot "after-$probe_lbl" "$PROBE_TARGET" $probe_default
    probe_delta "$PROBE_OUT/baseline-$probe_lbl.txt" "$PROBE_OUT/after-$probe_lbl.txt" \
        "$PROBE_OUT/delta-$probe_lbl.txt"
}

# probe_candidates <executable> <mechanism> <candidate|@none>...
#
# §8.4's question, which is about ACCEPTANCE and not about isolation: what is the smallest file content the
# agent accepts that sets no option? `agent-profile` has to create that file before the agent reads it, so
# the content is part of the mechanism. Aider set both the precedent and the cost of guessing — missing,
# empty and comment-only `.aider.conf.yml` each exit 2, and `{}` was accepted only because it was measured
# (`aider.rs:17-18`, SP2 design D5).
#
# One exit code per candidate, plus an index naming what each one held, because the transcript has to say
# which content the exit code belongs to.
probe_candidates() {
    probe_exe=$1
    probe_mech=$2
    shift 2
    probe_n=0
    : > "$PROBE_OUT/candidates.txt"
    for probe_cand in "$@"; do
        probe_n=$((probe_n + 1))
        # Each candidate is its own launch, and each launch initialises the agent's default location.
        # Restoring first keeps the sweep from deciding what a LATER behaviour step can still observe.
        probe_restore_known
        probe_prepare_target "$probe_cand"
        printf '%s: %s\n' "$probe_n" "$(printf '%s' "$probe_cand" | tr '\n' ' ')" \
            >> "$PROBE_OUT/candidates.txt"
        probe_apply "candidate-$probe_n" "$probe_exe" "$probe_mech" --version
    done
}
