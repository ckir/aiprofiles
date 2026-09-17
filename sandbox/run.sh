#!/bin/sh
# Runs a test workload in a disposable container and removes the container and its image afterwards.
#
# Usage: sandbox/run.sh test                      cargo nextest run --workspace on a copy of this checkout
#        sandbox/run.sh probe <agent>              sandbox/probes/<agent>.sh (installs the agent inside)
#        sandbox/run.sh [--net] shell              interactive shell, for writing or debugging a probe
#
# Engine: Podman when installed, otherwise Docker; override with AGENT_PROFILE_SANDBOX_ENGINE.
# `test` and `probe` have network (crates and agent installs need it); `shell` has none unless --net is given.
# Results: target/sandbox/<mode>[-<agent>]-<timestamp>/ holds build.log, exit-code, output.log (not for `shell`),
# diff.txt (the files the run added, changed or deleted inside the container) and whatever the workload writes
# to /out. A probe's `failures` and `steps` arrive there too, and so does `target/`, the profile directory
# the agent was pointed at — but none of the three is WRITTEN to /out. They live inside the container and
# are copied out after it stops: the bookkeeping because the measured party must not be able to forge the
# run's status, the profile directory because the agent is a second uid and the userns mapping admits one.
#
# Cleanup on a normal exit, Ctrl-C, TERM or HUP: the container and its image are removed. With local Podman every
# run also uses its own temporary image store under ${TMPDIR:-/var/tmp}, deleted at the end, so no image, layer or
# cache survives. A SIGKILL skips cleanup: remove a leftover store with `podman unshare rm -rf <store>`. Docker and
# remote Podman (Podman Desktop on macOS) keep the base image and build cache in their own store; see
# CONTRIBUTING.md.

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
        # AS THE HARNESS — before the install, with /out writable and /src readable — so an entire
        # evidence transcript could be authored with no agent misbehaving at all. The charset is the one
        # `resolve-agents.sh:93` and `transcript.sh:45` already enforce: a version that passed here but
        # not there could not name its own evidence file downstream.
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
    # The host user becomes the container's `probe` user, so /out is writable and its files stay yours.
    #
    # It maps ONE id, and the container now has two unprivileged users: `probe` the harness and `agent`
    # everything measured (`sandbox/Containerfile`). Anything `agent` wrote to this bind mount would
    # therefore land on the host as an UNMAPPED id. Nothing does: the agent's profile directory is
    # container-internal and is lifted out with `eng cp` after the container stops, below, and the
    # harness opens every /out descriptor itself before handing the agent the command. Widening the
    # mapping was the alternative and was rejected — an explicit --uidmap could carry a second id, but
    # keeping the second uid off the mount needs no mapping at all.
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
    # The Docker daemon does not map users: let the container's uid 1000 write the results directory.
    chmod a+rwx "$out"
fi

eng() {
    if [ -n "$store" ]; then
        podman --root "$store/root" --runroot "$store/run" "$@"
    else
        "$engine" "$@"
    fi
}

# shellcheck disable=SC2329 # invoked by the EXIT/INT/TERM/HUP traps below, which shellcheck does
# not follow. Removing it as "unused" would leave every run's container and image behind.
cleanup() {
    status=$?
    trap '' INT TERM HUP
    trap - EXIT
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

# The checkout is mounted read-only and copied without `target/`, so a run never writes to your tree.
#
# The copy is the harness's, and the agent is launched with it as the working directory — `sudo` does not
# change directory. So it is given the group `probe` and `agent` share, and made group-writable: an agent
# that writes beside its config (aider's `.aider.chat.history.md` lands in the working directory) would
# otherwise fail for lack of a writable cwd, and that failure would be recorded as the agent's behaviour
# rather than as the harness's setup.
copy='mkdir -p /home/probe/work && chgrp probe-share /home/probe/work && chmod 2775 /home/probe/work && tar -C /src --exclude=./target --exclude=./.clavity -cf - . | tar -C /home/probe/work -xf - && cd /home/probe/work'
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
    # shellcheck disable=SC2086 # $userns is empty or one flag
    eng run -it --init --name "$id" ${net:+--network "$net"} $userns --security-opt label=disable \
        --volume "$root:/src:ro" --volume "$out:/out" "$id" sh -c "$script"
else
    # --init forwards Ctrl-C to the workload, so an interrupted run stops and reaches cleanup.
    # shellcheck disable=SC2086
    eng run --init --name "$id" ${net:+--network "$net"} $userns --security-opt label=disable \
        --volume "$root:/src:ro" --volume "$out:/out" "$id" sh -c "$script" > "$out/output.log" 2>&1
fi
code=$(eng inspect --format '{{.State.ExitCode}}' "$id" 2>/dev/null || echo 125)
set -e
echo "$code" > "$out/exit-code"
# The harness's `failures` and `steps` are kept at /home/probe/.probe-state, off the writable /out mount,
# so the measured party cannot forge the run's status (`sandbox/probes/common.sh:23-37`). Lift them out
# now that the container has stopped, into the results directory where `transcript.sh:85` reads `steps`.
# Failure is expected and ignored: `shell` never sources the harness, and a container that died before it
# ran has no such directory. The container itself is removed by the EXIT trap, after this.
eng cp "$id:/home/probe/.probe-state/." "$out/" >/dev/null 2>&1 || true
# The profile directory the agent was pointed at, lifted out the same way and for a related reason. It
# does not live under /out either: `agent` is a SECOND uid and the userns mapping above admits one, so
# anything the agent wrote to the mount would land on the host as an unmapped id. Keeping it inside and
# copying it out afterwards means no second uid ever writes the mount. Kept for `probe` only — it is the
# only mode that has an agent — and, like the copy above, a run that died before creating it is fine.
if [ "$mode" = probe ]; then
    mkdir -p "$out/target"
    eng cp "$id:/home/agent/probe-target/." "$out/target/" >/dev/null 2>&1 || true
fi
eng diff "$id" > "$out/diff.txt" 2>&1 || true
[ "$mode" = shell ] || tail -n 20 "$out/output.log"
echo "sandbox: exit code $code" >&2
exit "$code"
