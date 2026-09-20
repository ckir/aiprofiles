# Shared steps for probe scripts. Sourced by sandbox/probes/<agent>.sh inside the sandbox container, from a
# copy of the checkout; results are written to container-internal directories that the measured party
# cannot reach, and sandbox/run.sh lifts them into target/sandbox/ once the container has stopped.
#
# A probe installs one agent, records its version and help text, and measures whether the agent's own
# isolation mechanism actually moves its files (SP4 design §7.3). It never launches the agent through
# `agent-profile`: the CLI refuses an agent word it does not know, and the nine SP4 agents are unknown to
# the binary at the commit that runs the probe.

set -eu

# Sourced by relative path, because the container runs a probe from the copied checkout's root
# (`sandbox/run.sh`'s `copy=` assignment cds to /home/probe/work) and so does the shell suite. A missing file
# here would otherwise surface as `probe_strip_ansi: not found` three steps later, naming a symptom instead
# of a cause.
if [ ! -f sandbox/probes/text.sh ]; then
    echo "probe: run from the repository root; sandbox/probes/text.sh is not here" >&2
    exit 2
fi
. sandbox/probes/text.sh

# Where the harness writes every artefact a transcript is assembled from: `<step>.cmd`, `<step>.txt`,
# `<step>.exit-code`, `version.extracted`, `strings.txt`, the snapshots, the deltas, `candidates.txt`.
# Overridable so the shell suite can run this file outside a container.
#
# CONTAINER-INTERNAL AND PROBE-OWNED, and that is the whole of the defence against a defect class no
# per-file guard reaches. Every artefact named above is written by a `>` performed by `probe` — and `>`
# follows a symlink. The agent therefore never has to write one of those files: it only has to create the
# PATH first, as a link. While this directory was `/out` — a read-write host mount that `run.sh`'s docker
# branch has to make world-writable, because that daemon maps no users — the agent could pre-create
# `delta-<label>.txt`, whose label is derived from the mechanism under test and so is public and
# deterministic, pointing it at `$PROBE_STATE/failures`. `probe_delta`'s own redirect then truncated the
# one file that decides the run's status, and an EMPTY delta is what an agent that changes nothing
# produces. Measured end-to-end in a container: `failures` 0 bytes, `probe_finish` exiting 0 with a failed
# step already recorded.
#
# `rm -f` before each redirect was rejected rather than overlooked: shell has no `O_NOFOLLOW`, so anything
# the install left running — a daemon an `npm install` script started — re-plants the link between the
# unlink and the open.
#
# MOVING THE DIRECTORY CLOSES THE CLASS, not the two instances of it that were found. With no harness
# write landing anywhere the agent can create an entry, there is no path to pre-create. `run.sh` lifts it
# into the results directory with `eng cp` after the container stops, exactly as it already does for
# $PROBE_STATE and the profile directory, so the transcript finds every artefact where it always was.
PROBE_OUT=${PROBE_OUT:-/home/probe/.probe-out}
# Where the harness keeps its own bookkeeping: `failures`, which alone decides the RUN'S STATUS, and
# `steps`, the ordered record of what ran and with what code. `steps` has never decided the status — it
# carries a non-zero row for a refused candidate, which is a measurement rather than a failure.
#
# A SEPARATE DIRECTORY from $PROBE_OUT, and it stays one now that $PROBE_OUT is container-internal too,
# because the two hold different kinds of thing: $PROBE_OUT holds what the run RECORDED, this holds what
# decides whether the run PASSED. `probe_finish` derives the run's exit status from `failures`, that status
# becomes the container's, the matrix job's, and the conclusion `verify-transcripts.sh` requires before it
# accepts a committed transcript as evidence — and §5.3's byte-comparison cannot see a forged one, because
# the uploaded artifact and the committed file both derive from the forged bytes. Keeping the two apart is
# what stops a defect in the artefact path from reaching the status, which is exactly the reach the planted
# delta symlink had while both lived on the same writable mount.
#
# This protects the BOOKKEEPING only. `help.txt`, `strings.txt` and every other recorded artefact are
# agent-produced and always were; what is restored is that the agent cannot control WHETHER its run is
# recorded as failed. `run.sh`'s `eng cp` block copies this directory into the results directory after the
# container exits, so `transcript.sh`'s `exit-codes:` block still finds `steps` where it has always been.
PROBE_STATE=${PROBE_STATE:-/home/probe/.probe-state}

# --- THE PRIVILEGE SPLIT --------------------------------------------------------------------------
#
# The container carries TWO unprivileged users (`sandbox/Containerfile`), because one is not a boundary:
# the two paragraphs above protect the artefacts and the bookkeeping by MOVING them, and a move only stops
# an accident. Code running at the harness's own uid can still write any path it knows, and the repository
# is public. What turns those moves into a boundary is that the measured party is a DIFFERENT UID, and so
# cannot write — or create a path inside — either directory.
#
#   probe  the harness. Owns $PROBE_OUT and $PROBE_STATE, both at mode 700. Snapshots, deltas, step and
#          failure bookkeeping, and the run's exit status.
#   agent  everything measured. The INSTALL runs here too: `npm install --global <pkg>` executes the
#          package's own install scripts, which are agent-controlled code running before the agent
#          binary ever launches, so an install as `probe` would hand an attacker the harness uid before
#          the boundary applied.
#
# WHICH HELPER RUNS AS WHICH USER. This enumeration is the boundary; a wrapper on the wrong line gives it
# away, and nothing downstream would notice. It is stated per helper because "the agent runs as agent" is
# not a rule a reader can apply to `probe_strings`, which runs `grep` OVER an agent file.
#
# HAND-MAINTAINED PROSE ASSERTING A MECHANICAL PROPERTY, which is a shape that rots: three separate reviews
# have now found a false sentence in this block. Half of it is mechanical from here on —
# `sandbox/tests/probe-harness.sh`'s "every function that crosses the privilege boundary is named in the
# enumeration" check reads this file, collects every function whose body calls `probe_as_agent` or
# `probe_record_agent`, and fails if any of them is missing below, so a new crossing cannot be added in
# silence. The other half is not and cannot be: a membership check cannot tell whether the REASON given
# beside a name is true, and a false reason is precisely what all three reviews found.
#
#   as `agent`, through probe_record_agent / probe_as_agent:
#     probe_npm_install, probe_uv_install, probe_script_install   the install and its install scripts
#     probe_version, probe_help                                   launches of the agent binary
#     probe_apply (all four mechanisms)                           the behaviour and candidate launches
#     probe_make_target's `mkdir`                                 the profile directory is the AGENT's:
#                                                                 created at the harness's uid it would
#                                                                 come back harness-owned and the party
#                                                                 that has to write into it could not.
#                                                                 Two call sites — once at start-up and
#                                                                 once per probe_prepare_target
#     probe_prepare_target's `rm`                                 destroying a tree the AGENT wrote: a
#                                                                 `rm -rf` as `probe` cannot unlink files
#                                                                 inside a subdirectory the agent created
#                                                                 at 755, and the harness must not need
#                                                                 more privilege than the party that
#                                                                 wrote it. (It recreates the tree through
#                                                                 probe_make_target, above; nothing here
#                                                                 chmods — the mode comes from `umask` at
#                                                                 creation, because chmod on a directory
#                                                                 you do not own is refused)
#     probe_prepare_target's candidate write                      the CONTENT is the harness's, but the
#                                                                 write crosses anyway, for the `tar -x`
#                                                                 reason below: $PROBE_TARGET must stay
#                                                                 agent-writable, so a link planted at the
#                                                                 candidate path is possible there in a
#                                                                 way it no longer is under $PROBE_OUT,
#                                                                 and a write performed at the agent's uid
#                                                                 can only reach what the agent could
#                                                                 already reach
#     probe_restore_default's rm, mkdir and `tar -x`              same, plus G5: an extraction that runs
#                                                                 as `agent` can only write where the
#                                                                 agent could already write, so a
#                                                                 symlink planted in the archive or on
#                                                                 the path reaches nothing new. The
#                                                                 archive is read from a descriptor the
#                                                                 harness opened, so $PROBE_STATE stays
#                                                                 unreadable to the agent.
#     probe_strings' `command -v`                                 "which file did the install put on the
#                                                                 PATH" is a question about the AGENT's
#                                                                 PATH. `command -v` is a shell builtin:
#                                                                 it reads a directory, it never runs
#                                                                 the agent.
#
#   as `probe`, and these must NOT be wrapped:
#     probe_record's bookkeeping      .cmd, .exit-code, `steps`, `failures` — the records the boundary
#                                     exists to protect. Only the command inside it crosses over.
#     probe_fail, probe_finish        the run's status.
#     probe_snapshot, probe_delta     the measurement itself, reading agent-owned locations through the
#                                     shared group.
#     probe_version's extraction      the harness's own statement about what it measured — the token
#     probe_candidates' index         SP4b types into the registry, and which content each candidate
#                                     exit code belongs to. NOTHING MECHANICAL STOPS A WRAPPER HERE, and
#                                     the claim that used to stand in this slot — that one "would fail
#                                     outright" because both land in $PROBE_OUT at 700 — was false. `>`
#                                     binds to the CALLING shell: `probe_as_agent printf … >
#                                     "$PROBE_OUT/version.extracted"` has the harness open the file as
#                                     `probe`, succeed, and hand the agent a descriptor. That is not an
#                                     accident of these two lines, it is exactly how probe_record
#                                     captures agent output. A wrapper here would therefore write the
#                                     file, at 700, with the AGENT choosing the bytes — the harness's
#                                     statement about what it measured, authored by the measured party.
#                                     What keeps these two on this side is this enumeration and review,
#                                     and probe-harness.sh's "every function that crosses the privilege
#                                     boundary is named in the enumeration" check backs only the
#                                     membership half of that.
#     probe_pristine's `tar -c`       writes into $PROBE_STATE, which the agent cannot reach.
#     probe_strings' readlink/find/grep   harness work reading an agent file. A `sudo` misplaced onto
#                                     THIS `find` would run agent-chosen paths at the harness uid and
#                                     hand the boundary away — it is the single most attractive mistake
#                                     in this file.
#
# THE ENVIRONMENT ACROSS THE SWITCH (G4). `sudo` resets the environment by default (`env_reset`, plus
# `secure_path` for the command lookup), so `PATH` and `NPM_CONFIG_PREFIX` do not survive the switch and
# every post-install launch would fail to find the binary the install just placed. The values are
# therefore passed EXPLICITLY, on the command line, through `env` — see probe_as_agent. Not `env_keep`:
# that puts the contract in /etc/sudoers.d where nothing in this repository reads it, and it rots
# silently. Not a login shell (`sudo -i`): that would run the agent's own dotfiles, which the install
# scripts can write, INSIDE the privilege switch — agent-controlled code choosing the environment the
# harness measures under. Explicit values also mean the harness decides what the agent runs with rather
# than exporting whatever its own environment happens to hold.

PROBE_AGENT_USER=${PROBE_AGENT_USER:-agent}
# THERE IS NO DEGRADED MODE. Every crossing goes through `sudo`, always, and a run in which the switch
# cannot be made is refused by the pre-flight below — before the first crossing, so before any agent code
# has run — and recorded as a `privilege` failure.
#
# There used to be one: with no `sudo` or no agent user, every crossing silently became a direct call at
# the harness's own uid. That was how `sandbox/tests/probe-harness.sh` ran this file on a developer's
# host, and it was the wrong trade. A run in that state has the measured party running as the harness,
# owning $PROBE_OUT and $PROBE_STATE, able to write anything a transcript is assembled from — worthless
# evidence that nonetheless produced a transcript indistinguishable from a real one. Recording the fact
# in the transcript does not rescue it: the record would be written into a directory the agent could
# then overwrite, in exactly the state the record exists to reveal. So the state is made unreachable
# instead, and the suite supplies a stub `sudo` of its own for every run.
#
# "The host has no sudo" was never a safe assumption to build on either: Windows 11 ships a `sudo.exe` in
# System32, which is on a Git Bash PATH, is a different program with different flags, and elevates
# through UAC. The old detection took the switch whenever BOTH a `sudo` and the named user resolved, so
# a suite run naming a real user with that `sudo.exe` first on the PATH would have invoked it. The suite
# now refuses to run unless `sudo` resolves to its own stub.

# The agent's home. Under two users `$HOME` is ambiguous — it is the HARNESS's home in every probe
# script, which is not where the agent writes — so the twelve probe scripts name the agent's default
# locations through this. Exported so it reaches harness-side children, not because the agent's own launch
# inherits it: every crossing goes through `sudo`'s `env_reset` (or the stub's `env -i`), which drops
# everything not passed explicitly, so the agent gets its home instead from `probe_as_agent`'s own `HOME=`.
#
# ONE DEFAULT, because there is one world. This used to fall back to the harness's own home whenever the
# boundary was absent, since a degraded launch passed no `HOME=` and so wrote under the harness's home.
# That branch was the site of the worst defect this harness has had — every probe watching the harness's
# home while the agent wrote its own, producing the empty delta `probe_delta` calls a launch that changed
# nothing, unambiguously — and with no degraded mode it has nothing left to serve.
#
# An explicit value still wins: `sandbox/tests/probe-harness.sh` points it at a scratch directory so a
# check can assert against a path no host actually has.
PROBE_AGENT_HOME=${PROBE_AGENT_HOME:-/home/agent}
export PROBE_AGENT_HOME
# The PATH the agent runs under: its own bin directories first, then the system ones. The harness's PATH
# does NOT contain these (`Containerfile`), so a launch that forgot to cross the boundary fails with 127
# instead of quietly running the agent at the harness's uid.
PROBE_AGENT_PATH=${PROBE_AGENT_PATH:-$PROBE_AGENT_HOME/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin}

# NOT under $PROBE_OUT, and now for the OPPOSITE reason to the one that moved $PROBE_OUT: this directory
# has to stay AGENT-WRITABLE, because the agent writes its own config into it, while $PROBE_OUT is
# probe-owned at 700 precisely so that nothing the agent can create sits in it. G1 holds as well: `agent`
# is a SECOND uid and `run.sh`'s `--userns=keep-id:uid=1000,gid=1000` maps exactly one, so anything
# `agent` wrote to a host bind mount would land there as an unmapped id. `run.sh` lifts this directory out
# with `eng cp` after the container stops, exactly as it does for $PROBE_STATE and $PROBE_OUT.
PROBE_TARGET=${PROBE_TARGET:-$PROBE_AGENT_HOME/probe-target}
# How long any one recorded command may run. A probe never waits for input; a hang is a recorded fact, not
# a job that burns its whole budget.
PROBE_TIMEOUT=${PROBE_TIMEOUT:-300}
# The version the maintainer asked for, or empty for "whatever the registry serves" (§7.4). `run.sh`'s
# `probe`-mode `script=` assignment passes it as the probe script's first argument — charset-checked at
# `run.sh`'s version-charset case arm and quoted there, so it reaches this file as one word — and a sourced
# file sees its caller's positional parameters, so no probe script has to thread it through.
PROBE_VERSION=${PROBE_VERSION:-${1:-}}
# The candidate file's name inside the profile directory. Some agents key on the extension — Continue
# reads a `config.yaml` and Amp a `settings.json` — so a probe that must name it can, and one whose agent
# does not care leaves it alone.
PROBE_CONFIG_NAME=${PROBE_CONFIG_NAME:-config}
# Whether a non-zero step is a FAILURE of the probe or a MEASUREMENT taken by it. Empty means failure,
# which is every step's default; `probe_candidates` sets it for the length of one launch.
#
# NOT read from the environment, unlike every setting above it. `run.sh` passes the container no `--env`,
# so nothing legitimate would arrive that way, and a variable that decides whether a failed step counts is
# the one thing an install script must not be able to preset.
PROBE_SOFT=
# Whether the command `probe_record` is about to run belongs to the measured party. Empty means the
# harness, which is every step's default; `probe_record_agent` sets it for the length of one call.
#
# NOT read from the environment, for PROBE_SOFT's reason and one more: this one decides WHICH UID a
# command runs at, so a value an install script could preset would be the boundary itself.
PROBE_AS_AGENT=

mkdir -p "$PROBE_OUT"
# 700 for $PROBE_STATE's reason and not a weaker one: every artefact here is written by a redirect the
# HARNESS performs, and a redirect into a directory the agent can create an entry in is a redirect the
# agent can aim. Nothing needs the agent to read this directory either — `probe_record` hands the agent
# open DESCRIPTORS, which is not directory access, and no path under $PROBE_OUT is ever passed to a
# command that crosses the boundary.
chmod 700 "$PROBE_OUT"
mkdir -p "$PROBE_STATE"
# The whole point, made explicit rather than left to the umask: `agent` cannot write what decides the
# run's status. `mkdir` would give 755 under the default umask, and group-readable state is state the
# measured party can read — and, once the shared group exists for the snapshot's sake, write.
chmod 700 "$PROBE_STATE"
: > "$PROBE_STATE/failures"
: > "$PROBE_STATE/steps"
# Where `probe_pristine` keeps the pre-launch copy of each watched default location — beside `failures`
# and `steps`, and for the same reason: the measured party must not be able to rewrite the state the
# harness restores it to. Cleared rather than merely created, because the archives are numbered from the
# index and a stale `1.tar` beside a fresh index would be restored over a location it was never taken
# from.
rm -rf "$PROBE_STATE/pristine"
mkdir -p "$PROBE_STATE/pristine"
: > "$PROBE_STATE/pristine/index"

# --- the privilege switch -------------------------------------------------------------------------

# probe_as_agent <command...>: run one command as the measured party.
#
# ALWAYS through `sudo`; there is no branch that runs the command directly (see "THERE IS NO DEGRADED
# MODE" above). `sandbox/tests/probe-harness.sh` reaches this line through a stub `sudo` that runs the
# command at the suite's own uid, so the suite exercises this exact argument vector but NOT the uid
# separation it exists for. Only a real container run measures that.
#
# The environment is passed explicitly because `sudo`'s `env_reset` strips it (G4, argued at THE
# PRIVILEGE SPLIT above). `env` itself is found through `secure_path`; everything after it is found
# through the PATH `env` has just set. `-n` because the rule is NOPASSWD and a probe must never block on
# a prompt: a misconfiguration has to fail, not hang until the step's timeout.
probe_as_agent() {
    sudo -n -u "$PROBE_AGENT_USER" -- env \
        "HOME=$PROBE_AGENT_HOME" \
        "PATH=$PROBE_AGENT_PATH" \
        "NPM_CONFIG_PREFIX=$PROBE_AGENT_HOME/.local" \
        "$@"
}

# probe_make_target: an empty profile directory, owned by the agent and writable by the harness.
#
# It is the AGENT's: created at the harness's uid it would come back harness-owned, and the agent — the
# party that has to write into it — could not. The harness still has to put the CANDIDATE file in it, so
# the directory needs a group both users are in and a group-write bit. The group comes from the setgid
# /home/agent (`Containerfile`), the same way everything else under it gets one, and the write bit from
# `umask` at creation. NOT a `chmod` afterwards: chmod on a directory you do not own is refused, and that
# refusal once aborted the probe at its first line with nothing said — measured in a container by pointing
# PROBE_AGENT_USER at a user that does not exist, back when that silently degraded the boundary instead of
# stopping the run at the pre-flight.
probe_make_target() {
    # shellcheck disable=SC2016 # `$1` is the INNER shell's argument; `umask` must apply to ITS mkdir.
    probe_as_agent sh -c 'umask 0002 && mkdir -p "$1"' probe_make_target "$PROBE_TARGET"
}

# probe_record <name> <command...>: run a command under a timeout, keep its output and exit code in
# $PROBE_OUT.
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
    #
    # AS THE VENDOR DOCUMENTS IT, without the privilege switch in front of it. `sudo -n -u agent -- env
    # HOME=... npm install ...` states the same fact about the agent plus a fact about this harness, and
    # only the first is evidence about the agent. That the install ran as `agent` is a property of the
    # harness, established by the boundary itself, not by a string the harness wrote about itself.
    printf '%s\n' "$*" > "$PROBE_OUT/$name.cmd"
    set +e
    # Both redirections are performed by THIS shell, as `probe`, before the switch, so what the agent is
    # handed is two open descriptors and no path it can act on. That is only half of what makes the write
    # the harness's, and the half that used to be missing is the DIRECTORY: a redirect into a directory
    # the agent can create an entry in is not a write the harness controls, because `>` follows a symlink
    # and the agent only has to get there first. $PROBE_OUT is probe-owned at 700, so it cannot — which is
    # what the claim reduces to now, rather than the aspirational one this comment used to make while
    # $PROBE_OUT was the world-writable /out mount. `timeout` runs on the far side so that the process it
    # signals is the agent's own, at the agent's uid.
    if [ -n "$PROBE_AS_AGENT" ]; then
        probe_as_agent timeout --kill-after=10s "$PROBE_TIMEOUT" "$@" > "$PROBE_OUT/$name.txt" 2>&1
    else
        timeout --kill-after=10s "$PROBE_TIMEOUT" "$@" > "$PROBE_OUT/$name.txt" 2>&1
    fi
    status=$?
    set -e
    echo "$status" > "$PROBE_OUT/$name.exit-code"
    # Appended in STEP ORDER, which a directory listing cannot reconstruct: sorted by name, `behaviour`
    # precedes `install`, and a transcript in that order reads as an agent launched before it existed.
    printf '%s %s\n' "$name" "$status" >> "$PROBE_STATE/steps"
    # `steps` records EVERY step and its code; `failures` records only what constitutes a probe FAILURE,
    # and those are not the same set. An acceptance sweep's refusals are the measurement (§8.4) — Aider
    # refuses a missing, an empty and a comment-only `.aider.conf.yml` by design (`aider.rs`'s
    # `INITIAL_CONFIG` constant) — so routing them here would make the probe exit non-zero, the matrix job
    # conclude failure, and `verify-transcripts.sh` refuse the transcript the sweep exists to produce.
    if [ "$status" -ne 0 ] && [ -z "$PROBE_SOFT" ]; then
        echo "$name $status" >> "$PROBE_STATE/failures"
    fi
    return 0
}

# probe_record_agent <name> <command...>: probe_record, with the command on the far side of the boundary.
#
# Two names rather than a `who` argument, because the call site is where the enumeration at THE PRIVILEGE
# SPLIT has to be readable: `probe_record_agent install npm ...` says which uid the install runs at
# without a reader tracing a variable. The bookkeeping inside is unchanged and stays with the harness —
# only the command crosses. `probe_record` always returns 0, so the flag is always cleared.
probe_record_agent() {
    PROBE_AS_AGENT=1
    probe_record "$@"
    PROBE_AS_AGENT=
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
#
# THE `harness-` PREFIX IS RESERVED, AND IT DECIDES WHETHER A RUN CAN BE EVIDENCE AT ALL. A step name
# beginning `harness-` means the failure was ours -- the image, this harness, or the operator who started
# the run -- and NOT a fact about the agent. `sandbox/transcript.sh` refuses to assemble a committable
# transcript for a run carrying one, because the alternative is evidence that blames a vendor for our
# breakage: a refused run records no version, so it is named `<id>-unknown.md`, which is exactly how a
# genuine "this agent would not install" transcript is named, and §5.3's verifier pairs any non-zero
# `probe-exit:` with a failed job and accepts it. MEASURED before this existed, end to end: a run refused
# for a missing agent user produced `example-unknown.md` and the verifier printed "all changed transcripts
# are bound to their runs".
#
# The prefix is the whole mechanism, so the three names carrying it are enumerated here and pinned by
# `sandbox/tests/probe-harness.sh`:
#
#   harness-privilege                 the pre-flight: no `sudo`, no agent user, or the rule does not reach
#   harness-version-refused           a version was passed to an agent installed by a vendor script
#   harness-mechanism-<step>          probe_apply was handed a mechanism it does not know
#
# WHAT IS DELIBERATELY NOT PREFIXED. `strings 127` is the agent's binary not being on its own PATH, which
# follows from the install and is a fact about the agent. `pristine-N` and `restore-N` are the harness
# failing to archive or restore a watched location -- but the vendor's install code runs BEFORE them and
# can cause exactly that, by chmod-ing its own profile directory, so calling them ours would hand an agent
# a way to disqualify its own measurement. Every `probe_record` row is the measured command's own status.
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

# The pre-flight, and IT MUST STAY AHEAD OF EVERY PRIVILEGED SWITCH — including the one that creates the
# profile directory, below. A `sudo` that is present but cannot reach `agent` otherwise surfaces as
# `command not found` in every step from the install onwards, a cascade that names a symptom twelve times
# and the cause never. Checked once, here, with the run stopped at the cause.
#
# The ORDER is the whole of it, and it was measured the wrong way round first: with the directory created
# before this check, a container whose sudoers rule had been removed died at that line under `set -e`,
# BEFORE `trap probe_finish EXIT` was installed — exit 1, an empty `steps`, an empty `failures` and not
# one word about why. Nothing may cross the boundary above this line.
#
# THE ONLY GATE, and it is unconditional: this is where a run with no usable boundary stops. Three causes,
# each named separately because each points at a different repair — no `sudo` in the image, no agent user
# in the image, or both present but the sudoers rule not letting one reach the other. All three are
# recorded as a `privilege` failure, so the transcript states the cause instead of carrying a run that
# never measured anything.
probe_privilege_refusal=
if ! command -v sudo >/dev/null 2>&1; then
    probe_privilege_refusal="there is no 'sudo' on the PATH"
elif ! id "$PROBE_AGENT_USER" >/dev/null 2>&1; then
    probe_privilege_refusal="there is no '$PROBE_AGENT_USER' user"
elif ! sudo -n -u "$PROBE_AGENT_USER" true >/dev/null 2>&1; then
    probe_privilege_refusal="'sudo -n -u $PROBE_AGENT_USER true' failed"
fi
if [ -n "$probe_privilege_refusal" ]; then
    echo "probe: $probe_privilege_refusal;" >&2
    echo "probe: the privilege boundary is not usable, so nothing measured here would be trustworthy" >&2
    probe_fail harness-privilege 1
    exit 1
fi

# The state a probe script sees before its first launch; every launch re-creates it. The FIRST privileged
# switch a probe makes, which is why it sits below the pre-flight rather than beside the other setup.
probe_make_target

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
    probe_record_agent install npm install --global "$@" "$probe_pkg${PROBE_VERSION:+@$PROBE_VERSION}"
}

# probe_uv_install <package> <python-version>: install one Python tool, at the requested version if given.
# The interpreter is pinned by the caller; CONTRIBUTING.md's "Pin interpreter versions an agent supports"
# guidance covers exactly this.
probe_uv_install() {
    probe_record_agent install uv tool install --python "$2" "$1${PROBE_VERSION:+==$PROBE_VERSION}"
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
        # `harness-version-refused`, not `install`: this is the OPERATOR asking for something this
        # installer cannot do, and `install` is the name a genuine vendor install failure records through
        # `probe_record_agent`. Sharing it made the two indistinguishable in `exit-codes:`.
        probe_fail harness-version-refused 2
        exit 2
    fi
    # The interpreter is the vendor's, because the script is the vendor's. Both script-installed agents
    # document `| bash`, and a bash script run under dash fails in ways that would be recorded as the
    # agent failing to install.
    probe_record_agent install sh -c "curl -fsSL '$1' | ${2:-sh}"
}

# --- Step 2: version ------------------------------------------------------------------------------

# probe_version <executable>: record `--version` verbatim, and beside it the one token SP4b types into
# the registry.
#
# The extraction rule is §7.3 step 2's: the first `[A-Za-z0-9._-]` run in the output that contains a
# digit — drawn from ONE LINE, not the whole capture. `probe_record` folds stderr into the same capture
# as stdout (`2>&1`), and ordinary startup noise on stderr sits ahead of the real version line and used
# to win outright: `(node:1234) [DEP0040] DeprecationWarning: ...` then `fakeagent 2.7.1` extracted
# `1234`, and a python traceback path `/usr/lib/python3.12/site-packages/x.py:41: SyntaxWarning: ...`
# then `fakeagent 0.86.2` extracted `python3.12`. Both sit inside Gate A's charset, so nothing downstream
# caught it, and nine of the twelve probes are npm installs, where an `(node:NNN) ...Warning` on stderr
# is routine — this was not a rare input.
#
# The line is chosen by: prefer the first line that names the executable itself (`$1`); otherwise fall
# back to the last non-empty line. Last-line-alone was considered and rejected — it breaks on an npm
# update-notifier banner, which prints AFTER the version line, so the executable-name preference has to
# run first. Requiring a dotted shape (`grep -E '[0-9]+\.[0-9]'`) was also rejected — it rejects `1234`
# but still accepts `python3.12`, so it would not have fixed the actual defect.
#
# It lives here rather than in a reviewer's head because Gate A resolves a path from the recorded
# version and SP4b resolves the same path from the registry; if the two derive the token differently, the
# gate fails on a file that exists. `aider --version` prints `aider 0.86.2` — a line, not a token, and the
# space alone violates Gate A's charset.
#
# The raw capture stays verbatim: it is the evidence. The extraction reads a stripped copy, because a
# coloured banner begins `ESC[0m`, whose `0m` is a digit-bearing run that Gate A would happily accept.
probe_version() {
    probe_record_agent version "$1" --version
    probe_stripped=$(probe_strip_ansi < "$PROBE_OUT/version.txt")
    probe_vline=$(printf '%s\n' "$probe_stripped" | grep -F -m1 -- "$1" || true)
    if [ -z "$probe_vline" ]; then
        probe_vline=$(printf '%s\n' "$probe_stripped" | grep -v '^[[:space:]]*$' | tail -n1 || true)
    fi
    probe_extracted=$(
        printf '%s\n' "$probe_vline" \
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
    probe_record_agent help "$1" --help
}

# --- Step 4: strings ------------------------------------------------------------------------------

# probe_strings <executable> [documented-variable...]
#
# Two questions, one file: does each DOCUMENTED credential variable actually appear in what was installed,
# and what UNDOCUMENTED ones appear beside them? The second is the one that pays. Claude's adapter cites
# `CLAUDE_CODE_USE_BEDROCK` and three OAuth variables as "binary strings measured" in `claude.rs`'s evidence
# `notes` field — no page documented them, and each one is a way a user defeats the isolation the adapter
# promises.
#
# There is no `strings(1)` in the image: `sandbox/Containerfile` installs no binutils. `grep -a` reads a
# binary as text and is already a dependency, so this uses that rather than growing the image.
#
# THE ONE HELPER THAT STRADDLES THE BOUNDARY, and the enumeration at the top of this file exists largely
# for it. It is HARNESS work — it reads an agent file and writes the harness's record of it — so the
# readlink, the find and the greps below stay at the harness's uid and must never be wrapped. Only the
# path RESOLUTION crosses, because "which file did the install put on the PATH" is a question about the
# AGENT's PATH, which the harness deliberately does not carry (`Containerfile`). `command -v` is a shell
# builtin: it reads a directory, it never executes the agent.
probe_strings() {
    probe_exe=$1
    shift
    probe_out_file=$PROBE_OUT/strings.txt
    # shellcheck disable=SC2016 # `$1` is the INNER shell's argument, not this one's. Expanding it here
    # would resolve the executable against the harness's PATH, which is the one answer this must not give.
    probe_resolved=$(probe_as_agent sh -c 'command -v "$1" 2>/dev/null || true' probe_strings "$probe_exe" || true)
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
#
# HARNESS WORK, at the harness's uid, and it must stay that way: this is the measurement, and a
# measurement taken by the party being measured is not one. Both locations it walks are agent-owned now,
# so the reading depends on the shared group and the setgid /home/agent the Containerfile sets up — not
# on borrowing the agent's privileges. `find` does not follow symlinks without `-L` and none is passed,
# so a link planted in either location is LISTED rather than descended into.
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
            # (`aider.rs`'s `FILE_NAME` constant). Reporting an existing file as `(absent)` would read as the
            # agent having written nothing to its default location, which is the finding the whole probe is for.
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
# MISSING `.aider.conf.yml` exactly as it refuses an empty one (`aider.rs`'s `INITIAL_CONFIG` constant),
# and an adapter that creates nothing would hit that.
#
# The removal runs AS THE AGENT. What is being removed is whatever the agent last wrote into its own
# profile directory, and a `rm -rf` at the harness's uid cannot unlink files inside a subdirectory the
# agent created at mode 755 — the harness would need write access to agent-owned directories it has no
# business writing. At the agent's uid it needs no such access and can reach nothing the agent could not
# already reach.
#
# THE CANDIDATE WRITE RUNS AS THE AGENT TOO, although the content is the harness's. Every other harness
# write was taken out of reach by moving $PROBE_OUT; this one cannot move, because $PROBE_TARGET has to
# stay agent-writable — the agent writes its own config there, which is the thing being measured. So the
# path is one the agent can pre-create as a symlink, between `probe_make_target` and the line below, and a
# `>` performed by `probe` would follow it: an empty candidate then truncates whatever it points at, and
# `probe_finish` keys on `failures` being non-empty. Crossing the boundary is the structural answer, the
# same one `probe_restore_default`'s `tar -x` gives — a write performed at the agent's uid can only reach
# what the agent could already reach, so the link gains nothing. `rm -f` first is not an answer: shell has
# no `O_NOFOLLOW`, so the link comes back between the unlink and the open. `set -C` was the alternative —
# it fails loudly on a planted symlink — and was rejected for making the write non-idempotent while
# resting on the shell's clobber check rather than on who is doing the writing.
#
# The `umask` still makes the file group-writable, so an agent that rewrites its own config in place can,
# rather than failing with EACCES and having that recorded as the mechanism refusing the content.
probe_prepare_target() {
    probe_as_agent rm -rf "$PROBE_TARGET"
    probe_make_target
    case "${1:-@none}" in
        @none) ;;
        *)
            # shellcheck disable=SC2016 # `$1` and `$2` are the INNER shell's arguments; both the umask
            # and the redirect have to happen on the far side of the switch.
            probe_as_agent sh -c 'umask 0002 && printf %s "$1" > "$2"' \
                probe_prepare_target "$1" "$PROBE_TARGET/$PROBE_CONFIG_NAME"
            ;;
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
        probe_slot=$(($(wc -l < "$PROBE_STATE/pristine/index") + 1))
        printf '%s %s\n' "$probe_slot" "$probe_loc" >> "$PROBE_STATE/pristine/index"
        if [ ! -e "$probe_loc" ]; then
            # A location the agent has not created yet. Its ABSENCE is the state to restore, and a marker
            # records it: "registered and absent" and "never registered" must not look the same to
            # `probe_restore_default`, or a failed archive would read as absence and delete a real one.
            : > "$PROBE_STATE/pristine/$probe_slot.absent"
        # The archive is taken AS THE HARNESS and lands in $PROBE_STATE, which the agent cannot read or
        # write: it is the state probe_restore_default restores TO, so the measured party must not be
        # able to choose it. It reads the agent's files through the shared group, which is why the
        # Containerfile gives /home/agent one; a location the agent has chmodded to 700 is unreadable
        # and lands here as a recorded `pristine-N` failure rather than as a silently empty archive.
        elif ! tar -cf "$PROBE_STATE/pristine/$probe_slot.tar" \
            -C "$(dirname "$probe_loc")" "$(basename "$probe_loc")" 2>/dev/null; then
            rm -f "$PROBE_STATE/pristine/$probe_slot.tar"
            echo "probe: could not archive $probe_loc before the first launch" >&2
            probe_fail "pristine-$probe_slot" 1
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
        # THE WRITING HALF RUNS AS THE AGENT; the archive stays the harness's. Three reasons, and the
        # first two are the same ones probe_prepare_target gives: the tree being removed is the agent's,
        # and the restored tree has to be WRITABLE BY THE AGENT afterwards — extracted at the harness's
        # uid it would come back harness-owned, and the next launch could not rewrite its own config.
        # The third is G5. A `tar -x` that writes through a symlink — one planted on the path between the
        # two steps, or carried in the archive because the agent planted it in the location before the
        # archive was taken — can then only write where the agent could already write. That is a
        # structural answer rather than a flag: it does not depend on which hardening a given tar
        # implements. The archive itself is read from a descriptor THIS shell opened, so $PROBE_STATE
        # stays unreadable to the agent; that also keeps tar off the caller's stdin, which the loops in
        # probe_restore_known and probe_pristine_id are reading the index from.
        probe_as_agent rm -rf "$probe_rloc"
        probe_as_agent mkdir -p "$(dirname "$probe_rloc")"
        if ! probe_as_agent tar -xf - -C "$(dirname "$probe_rloc")" \
            < "$PROBE_STATE/pristine/$probe_rn.tar"; then
            echo "probe: could not restore $probe_rloc from its pristine copy" >&2
            probe_fail "restore-$probe_rn" 1
        fi
    elif [ -f "$PROBE_STATE/pristine/$probe_rn.absent" ]; then
        probe_as_agent rm -rf "$probe_rloc"
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
            probe_record_agent "$probe_name" env "${probe_mech#env:}=$PROBE_TARGET" "$probe_exe" "$@" ;;
        envfile:*)
            probe_record_agent "$probe_name" env "${probe_mech#envfile:}=$PROBE_TARGET/$PROBE_CONFIG_NAME" "$probe_exe" "$@" ;;
        flagdir:*)
            probe_record_agent "$probe_name" "$probe_exe" "${probe_mech#flagdir:}" "$PROBE_TARGET" "$@" ;;
        flagfile:*)
            probe_record_agent "$probe_name" "$probe_exe" "${probe_mech#flagfile:}" "$PROBE_TARGET/$PROBE_CONFIG_NAME" "$@" ;;
        *)
            echo "probe: unknown mechanism $probe_mech" >&2
            # A probe script naming a mechanism this file does not implement is OUR bug, not the agent's.
            probe_fail "harness-mechanism-$probe_name" 2
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
# (`aider.rs`'s `INITIAL_CONFIG` constant, SP2 design D5).
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
        # A REFUSAL IS THE ANSWER, not an error. The sweep asks which contents the agent accepts, and
        # three of Aider's four are designed to be rejected; a rejection therefore lands in `candidate-N
        # .exit-code` and in `steps` — the transcript states every one of them — but not in `failures`.
        # Scoped to the launch itself, so an unknown mechanism, a failed restore, or anything else this
        # loop does still fails the probe the way it always did.
        PROBE_SOFT=1
        probe_apply "candidate-$probe_n" "$probe_exe" "$probe_mech" --version
        PROBE_SOFT=
    done
}
