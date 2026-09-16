# GitHub Copilot CLI probe: `sandbox/run.sh probe copilot [version]`.
#
# Package @github/copilot (registry-verified 2026-09-16; `bin` is `copilot`), default `$HOME/.copilot`
# and mechanism COPILOT_HOME, both from
# https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference
#
# The three token variables have a documented precedence -- COPILOT_GITHUB_TOKEN, then GH_TOKEN, then
# GITHUB_TOKEN -- so all three defeat isolation and D11 makes the adapter declare each.
. sandbox/probes/common.sh

probe_npm_install @github/copilot
probe_version copilot
probe_help copilot
probe_strings copilot COPILOT_GITHUB_TOKEN GH_TOKEN GITHUB_TOKEN COPILOT_HOME
probe_behaviour copilot "$HOME/.copilot" env:COPILOT_HOME @none
