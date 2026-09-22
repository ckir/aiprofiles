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
# NO LAUNCH OVERRIDE, and the empty delta it produces is the honest measurement rather than a gap. This
# file previously launched `plugin list` and claimed it "writes into the target". THAT CLAIM WAS FALSE,
# and the way it became false is worth keeping, because it can be made again by anyone measuring an agent
# outside this harness.
#
# The original measurement wiped the home directory before each launch. copilot extracts a runtime
# package into `$PROBE_AGENT_HOME/.cache/copilot` on first use -- it announces it, "Package extraction
# took ..." -- and THE EXTRACTION writes a log into COPILOT_HOME. With a wiped home every launch is a
# first use, so every command appeared to write, and `plugin list` got the credit. Under this harness the
# cache survives from the `version` and `help` steps above, nothing is extracted, and `plugin list`
# writes nothing at all. Re-measured against the real harness: `plugin list` and `mcp list` both leave
# the target empty.
#
# So the general rule, which cost two agents a wrong launch command: WIPING THE HOME DIRECTORY BETWEEN
# MEASUREMENTS CONFLATES "what this command writes" WITH "what first-run initialisation writes". A
# discovery run that wipes gives an upper bound on the delta, never the delta.
#
# `init` IS NOT THE ANSWER EITHER, though it does write a log to COPILOT_HOME. Its actual job is to write
# `.github/` into the CURRENT WORKING DIRECTORY, and this harness runs the agent with the HARNESS's cwd,
# which the agent cannot write: it exits 1 with `EACCES ... mkdir '/home/probe/.github'`. Declaring that
# code with PROBE_EXPECT would bake a fact about this harness's cwd into a measurement that is supposed
# to be about the agent -- the exact confusion PROBE_EXPECT's own comment warns against.
#
# The default-location watch stays `~/.copilot`. `$PROBE_AGENT_HOME/.cache/copilot` holds the extracted
# runtime -- measured at 283 paths, and COPILOT_HOME does not move it -- but that is a cache keyed by
# version and platform, not configuration or credentials. Same call as opencode's `.cache` directory.
probe_behaviour copilot "$PROBE_AGENT_HOME/.copilot" env:COPILOT_HOME @none
