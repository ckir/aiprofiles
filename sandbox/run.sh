#!/bin/sh
# Runs a test workload in a disposable container and removes the container and its image afterwards.
#
# Usage: sandbox/run.sh test                      cargo nextest run --workspace on a copy of this checkout
#        sandbox/run.sh probe <agent>              sandbox/probes/<agent>.sh (installs the agent inside)
#        sandbox/run.sh [--net] shell              interactive shell, for writing or debugging a probe
#
# Engine: Podman when installed, otherwise Docker; override with AGENT_PROFILE_SANDBOX_ENGINE.
# `test` and `probe` have network (crates and agent installs need it); `shell` has none unless --net is given.
# Results: target/sandbox/<mode>[-<agent>]-<timestamp>/ holds build.log, exit-code, output.log (not for `shell`)
# and diff.txt (the files the run added, changed or deleted inside the container). A probe's artefacts, its
# `failures` and `steps`, and `target/` — the profile directory the agent was pointed at — arrive there too,
# and NONE of them is written to a host mount: they live inside the container on directories the measured
# party cannot reach, and are copied out after it stops (see the `eng cp` block inside `cleanup`). Only
# `shell`, which has no agent in it, still mounts the results directory at /out.
#
# Cleanup on a normal exit, Ctrl-C, TERM or HUP: the artefacts are lifted out and then the container and its image
# are removed — in that order, so an install interrupted after ten minutes still leaves its evidence behind. With
# local Podman every run also uses its own temporary image store under ${TMPDIR:-/var/tmp}, deleted at the end, so
# no image, layer or cache survives. A SIGKILL skips cleanup: remove a leftover store with
# `podman unshare rm -rf <store>`. Docker and remote Podman (Podman Desktop on macOS) keep the base image and build
# cache in their own store; see CONTRIBUTING.md.

set -eu

usage() {
    sed -n '/^# Usage:/,/^#$/p' "$0" | sed '/^#$/d; s/^# \{0,1\}//' >&2
    exit 2
}

net=none
if [ "${1:-}" = "--net" ]; then
    net=
    shift
fi
[ $# -ge 1 ] || usage
mode=$1
shift
agent=
case "$mode" in
    test) [ $# -eq 0 ] || usage; net= ;;
    shell) [ $# -eq 0 ] || usage ;;
    probe)
        # An optional second argument pins the agent version, so a measurement can be repeated.
        [ $# -eq 1 ] || [ $# -eq 2 ] || usage
        version=${2:-}
        net=
        agent=$1
        case "$agent" in
            '' | *[!a-z0-9-]*) echo "sandbox: agent names are [a-z0-9-]" >&2; exit 2 ;;
        esac
        # The version is spliced into the container's shell command below, so it is charset-checked
        # exactly like the agent word beside it. Without this, `probe claude '1.0; cmd; echo'` ran `cmd`
        # AS THE HARNESS — before the install, with every recorded artefact writable — so an entire
        # evidence transcript could be authored with no agent misbehaving at all. The charset is the one
        # `resolve-agents.sh`'s and `transcript.sh`'s own `*[!A-Za-z0-9._-]*` version-charset case arms
        # already enforce: a version that passed here but not there could not name its own evidence file downstream.
        case "$version" in
            '') ;;
            *[!A-Za-z0-9._-]*) echo "sandbox: versions are [A-Za-z0-9._-]" >&2; exit 2 ;;
        esac
        ;;
    *) usage ;;
esac

root=$(cd "$(dirname "$0")/.." && pwd)
if [ "$mode" = probe ] && [ ! -f "$root/sandbox/probes/$agent.sh" ]; then
    echo "sandbox: no probe script sandbox/probes/$agent.sh" >&2
    exit 2
fi

engine=${AGENT_PROFILE_SANDBOX_ENGINE:-}
if [ -z "$engine" ]; then
    if command -v podman >/dev/null 2>&1; then engine=podman; else engine=docker; fi
fi
case "$engine" in
    podman | docker) ;;
    *) echo "sandbox: AGENT_PROFILE_SANDBOX_ENGINE must be podman or docker" >&2; exit 2 ;;
esac
command -v "$engine" >/dev/null 2>&1 || { echo "sandbox: $engine is not installed" >&2; exit 2; }

stamp=$(date +%Y%m%d-%H%M%S)
id="agent-profile-sandbox-$stamp-$$"
out="$root/target/sandbox/$mode${agent:+-$agent}-$stamp"
mkdir -p "$out"

store=
userns=
if [ "$engine" = podman ]; then
    # The host user becomes the container's `probe` user, so `shell`'s /out mount is writable and its
    # files stay yours.
    #
    # It maps TWO ids, and not to what their numbers might suggest. The container has two unprivileged
    # users: `probe` the harness and `agent` everything measured (`sandbox/Containerfile`). `uid=1000` maps
    # the host user to `probe`, but `gid=1000` does NOT map to `probe`'s own group — that one is gid 1001 —
    # it maps to `probe-share`, the supplementary group both `probe` and `agent` belong to. Anything
    # `agent` wrote to a bind mount still lands on the host as UNMAPPED, in both ids: its uid is 1001, and
    # its files take its own primary group, 1002, because no bind mount carries the setgid bit that gives
    # everything under /home/agent the shared group. Nothing does write one, and nothing can: `probe` and
    # `test` mount no results directory at all, `/src` is mounted `:ro` in every mode, and everything a
    # probe produces is written container-internally and lifted out with `eng cp` afterwards. Widening the
    # mapping was the alternative and was rejected — an explicit --uidmap could carry a second id, but
    # keeping the second uid off the mount needs no mapping.
    #
    # WHICH HALF IS MEASURED. The ids are: running this Containerfile's own `groupadd`/`useradd` lines on
    # its pinned base image gives `uid=1000(probe) gid=1001(probe) groups=1001(probe),1000(probe-share)`
    # and `uid=1001(agent) gid=1002(agent) groups=1002(agent),1000(probe-share)` — `groupadd probe-share`
    # runs first, so it takes 1000 and `probe`'s own group is pushed to 1001. What is NOT measured is what
    # podman then does with that mapping: podman is not installed on the maintainer's host, so the flag
    # below has never been executed there.
    userns=--userns=keep-id:uid=1000,gid=1000
    # Remote Podman (a Podman machine, as on macOS) has no --root/--runroot; it keeps its own store.
    if ! remote=$(podman info --format '{{.Host.ServiceIsRemote}}' 2>&1); then
        echo "sandbox: podman info failed: $remote" >&2
        exit 2
    fi
    if [ "$remote" = true ]; then
        echo "sandbox: remote Podman: the base image and build cache stay in the Podman machine" >&2
    else
        store=$(mktemp -d "${TMPDIR:-/var/tmp}/agent-profile-sandbox.XXXXXX")
    fi
else
    # The Docker daemon does not map users, and only `shell` still mounts this directory (see `eng run`).
    [ "$mode" != shell ] || chmod a+rwx "$out"
fi

eng() {
    if [ -n "$store" ]; then
        podman --root "$store/root" --runroot "$store/run" "$@"
    else
        "$engine" "$@"
    fi
}

# Invoked by the EXIT/INT/TERM/HUP traps below, which shellcheck does not follow. Removing it as "unused"
# would leave every run's container and image behind.
#
# BOTH codes below, because the two shellchecks in play disagree about which one this is. 0.11 (the
# maintainer's) reports the FUNCTION as never invoked, SC2329. The older build on `ubuntu-latest`, which is
# what the Shellcheck job actually runs, reports every COMMAND inside it as unreachable, SC2317 -- so a
# tree that passes locally fails in CI, which is what happened on this branch's first push.
#
# The directive is the LAST line before the function on purpose: a directive binds to the next command,
# and putting explanation between the two is what made the first attempt at this fix fail in CI again.
# shellcheck disable=SC2317,SC2329
cleanup() {
    status=$?
    trap '' INT TERM HUP
    trap - EXIT
    # EVERYTHING A PROBE PRODUCED LIVES INSIDE THE CONTAINER, and is lifted into the results directory here
    # — IN THE TRAP, ahead of the `eng rm` below, because the trap is the only place that runs on BOTH exits.
    # On the main path it once did, and a Ctrl-C during a multi-minute agent install then reached `exit 130`,
    # `cleanup`, and `eng rm -f` with the copies still pending: the container went away with every artefact
    # in it. Measured on the host against a stub engine, before this moved: an interrupted run logged
    # `build, run, rm, rmi` and no `cp` at all, and its results directory held only build.log and an empty
    # output.log. It fails closed rather than wrong — `$out/exit-code` is not written either, so
    # `transcript.sh` reads `probe_exit=unknown` instead of a truncated transcript — but the evidence a long
    # run bought is gone. Nothing here is a second copy: the main path no longer copies at all, and `cleanup`
    # runs once (`trap - EXIT` above).
    #
    # The engine end of it was measured too, against Docker rather than reasoned about: `cp <id>:<dir>/.` reads
    # a RUNNING container as happily as a stopped one — which is what an interrupt can leave, since the copy
    # may start before the workload has finished stopping — overwrites the files it carries, adds the ones it
    # does not, and leaves anything else in the destination alone, so running it twice changes nothing. Against
    # a REMOVED container it fails with "No such container", which is precisely the old order.
    #
    # Three directories, three reasons — and the first two are why `probe` and `test` no longer mount /out at
    # all. A writable host directory inside a container the measured party shares is not just somewhere it can
    # drop files; it is somewhere it can PRE-CREATE A PATH, and `transcript.sh` reads this directory
    # afterwards. The copy-back overwrites what it carries but cannot unwrite what it does not.
    #
    #   /home/probe/.probe-out     the artefacts a transcript is assembled from. Probe-owned at 700, because
    #                              every one of them is written by a `>` the HARNESS performs, and `>` follows
    #                              a symlink: under /out the agent could aim `delta-<label>.txt` at the
    #                              bookkeeping and an empty delta truncated it (`sandbox/probes/common.sh`, at
    #                              PROBE_OUT — measured in a container).
    #   /home/probe/.probe-state   `failures` and `steps`: the run's status, which the measured party must not
    #                              be able to forge. Copied last, so the bookkeeping is the last word.
    #   /home/agent/probe-target   the profile directory the agent was pointed at. `agent` is a SECOND uid and
    #                              the userns mapping above admits one, so anything IT wrote to a host mount
    #                              would land there as an unmapped id. `probe` only: it is the only mode that
    #                              has an agent.
    #
    # Failure is expected and ignored: `shell` never sources the harness, a container that died before it ran
    # has no such directory, and a build that never produced a container has no container. `eng diff` is NOT
    # here — it needs the container alive, and it still runs on the main path, which is the only path that
    # reaches a stopped container rather than an interrupted one.
    eng cp "$id:/home/probe/.probe-out/." "$out/" >/dev/null 2>&1 || true
    eng cp "$id:/home/probe/.probe-state/." "$out/" >/dev/null 2>&1 || true
    if [ "$mode" = probe ]; then
        mkdir -p "$out/target"
        eng cp "$id:/home/agent/probe-target/." "$out/target/" >/dev/null 2>&1 || true
    fi
    eng rm -f "$id" >/dev/null 2>&1 || true
    eng rmi -f "$id" >/dev/null 2>&1 || true
    if [ -n "$store" ]; then
        eng unshare rm -rf "$store" >/dev/null 2>&1 || rm -rf "$store" >/dev/null 2>&1 || true
        if [ -e "$store" ]; then echo "sandbox: could not delete $store" >&2; fi
    fi
    echo "sandbox: removed container and image $id${store:+ and image store $store}; results in $out" >&2
    exit "$status"
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
trap 'exit 129' HUP

arch=$(uname -m)
case "$arch" in
    x86_64 | amd64) nextest_platform=linux ;;
    aarch64 | arm64) nextest_platform=linux-arm ;;
    *) echo "sandbox: unsupported architecture $arch" >&2; exit 2 ;;
esac

if ! eng build --tag "$id" --build-arg "NEXTEST_PLATFORM=$nextest_platform" \
    --file "$root/sandbox/Containerfile" "$root/sandbox" > "$out/build.log" 2>&1; then
    echo 125 > "$out/exit-code"
    tail -n 20 "$out/build.log" >&2
    echo "sandbox: image build failed; see $out/build.log" >&2
    exit 125
fi

# The checkout is mounted read-only and copied without `target/`, `.clavity` or `.git`, so a run never
# writes to your tree.
#
# `.git` is excluded as hygiene, not as speed — it is a couple of megabytes. Nothing in the container reads
# it: `cargo nextest run --workspace` needs no history, and `crates/agent-profile/tests/resolution.rs` says
# in its module comment that its repositories are "synthetic `.git` layouts in guarded temp directories, so
# no test needs `git` and none discovers this checkout". What the exclusion removes is a copy of every
# commit, branch and remote in the working directory an agent is launched in.
#
# The copy is the harness's, and the agent is launched with it as the working directory — `sudo` does not
# change directory. So it is given the group `probe` and `agent` share, and made group-writable: an agent
# that writes beside its config (aider's `.aider.chat.history.md` lands in the working directory) would
# otherwise fail for lack of a writable cwd, and that failure would be recorded as the agent's behaviour
# rather than as the harness's setup.
copy='mkdir -p /home/probe/work && chgrp probe-share /home/probe/work && chmod 2775 /home/probe/work && tar -C /src --exclude=./target --exclude=./.clavity --exclude=./.git -cf - . | tar -C /home/probe/work -xf - && cd /home/probe/work'
case "$mode" in
    test) script="$copy && cargo nextest run --workspace --no-tests=pass" ;;
    probe)
        # `${version:+ ...}` appends the argument only when there IS one: measured, an unconditional
        # `'$version'` passes one EMPTY argument for an absent version where today's form passes none.
        #
        # shellcheck disable=SC2016 # the single quotes are literal on purpose. They are for the
        # container's `sh -c "$script"`, which has to see the version as one word; `$version` itself
        # still expands here, because the quotes sit inside a double-quoted string.
        script="$copy && sh sandbox/probes/$agent.sh${version:+ '$version'}"
        ;;
    shell) script="$copy && exec bash" ;;
esac

set +e
if [ "$mode" = shell ]; then
    # shellcheck disable=SC2086 # $userns is empty or one flag. /out is mounted HERE ONLY; see `eng cp`.
    eng run -it --init --name "$id" ${net:+--network "$net"} $userns --security-opt label=disable \
        --volume "$root:/src:ro" --volume "$out:/out" "$id" sh -c "$script"
else
    # --init forwards Ctrl-C to the workload, so an interrupted run stops and reaches cleanup.
    # shellcheck disable=SC2086
    eng run --init --name "$id" ${net:+--network "$net"} $userns --security-opt label=disable \
        --volume "$root:/src:ro" "$id" sh -c "$script" > "$out/output.log" 2>&1
fi
code=$(eng inspect --format '{{.State.ExitCode}}' "$id" 2>/dev/null || echo 125)
set -e
echo "$code" > "$out/exit-code"
# The container is still alive here and `eng diff` needs it to be; the artefacts are lifted out by the EXIT
# trap afterwards (see the `eng cp` block in `cleanup`). `output.log` is written by THIS shell's redirect of
# `eng run`, not by the copy-back, so tailing it before the copies shows exactly what it always showed.
eng diff "$id" > "$out/diff.txt" 2>&1 || true
[ "$mode" = shell ] || tail -n 20 "$out/output.log"
echo "sandbox: exit code $code" >&2
exit "$code"
