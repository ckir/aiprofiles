# Shared steps for probe scripts. Sourced by sandbox/probes/<agent>.sh inside the sandbox container, from a
# copy of the checkout; results go to /out, which sandbox/run.sh copies back to target/sandbox/.
#
# A probe installs one agent, records its version and help text, and runs agent-profile's dry run against it
# with a throwaway application root, so adapter evidence (spec §28) can be refreshed from a clean install.

set -eu

export AGENT_PROFILE_HOME=/home/probe/agent-profile-home

# probe_record <name> <command...>: run a command, keep its output and exit code in /out.
probe_record() {
    name=$1
    shift
    set +e
    "$@" > "/out/$name.txt" 2>&1
    echo "$?" > "/out/$name.exit-code"
    set -e
}

# probe_agent_profile <agent>: build agent-profile and record its dry run for <agent> with profile `probe`.
probe_agent_profile() {
    cargo build --quiet --bin agent-profile
    probe_record agent-profile-version target/debug/agent-profile --version
    probe_record dry-run target/debug/agent-profile "$1" probe --dry-run
}
