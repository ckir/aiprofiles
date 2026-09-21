# GitHub Copilot CLI probe: `sandbox/run.sh probe copilot [version]`.
#
# Package @github/copilot (registry-verified 2026-09-16; `bin` is `copilot`), default
# `$PROBE_AGENT_HOME/.copilot` and mechanism COPILOT_HOME, both from
# https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference
#
# The three token variables have a documented precedence -- COPILOT_GITHUB_TOKEN, then GH_TOKEN, then
# GITHUB_TOKEN -- so all three defeat isolation and D11 makes the adapter declare each.
. sandbox/probes/common.sh

probe_npm_install @github/copilot
probe_version copilot
probe_help copilot
probe_strings copilot COPILOT_GITHUB_TOKEN GH_TOKEN GITHUB_TOKEN COPILOT_HOME
# `plugin list` RATHER THAN THE DEFAULT `--version`, which was measured to write nothing at all -- an
# empty delta that says nothing about COPILOT_HOME. `plugin list` exits 0, needs no credentials, and
# writes into the target. `init` writes more (a session-state tree as well as logs) but exits 1 without
# authentication, so it would record a failure for a measurement that worked.
#
# The default-location watch stays `~/.copilot` deliberately. Every launch also extracts a runtime
# package into `$PROBE_AGENT_HOME/.cache/copilot` -- measured at 283 paths, and COPILOT_HOME does not
# move it -- but that is a cache keyed by version and platform, not configuration or credentials, and
# watching it would bury the delta that matters. Same call as opencode's `.cache` directory.
probe_behaviour copilot "$PROBE_AGENT_HOME/.copilot" env:COPILOT_HOME @none plugin list
