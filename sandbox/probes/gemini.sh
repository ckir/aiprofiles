# Gemini CLI probe: `sandbox/run.sh probe gemini [version]`.
#
# Package @google/gemini-cli (registry-verified 2026-09-16; `bin` is `gemini`), default
# `$PROBE_AGENT_HOME/.gemini` (https://geminicli.com/docs/reference/configuration/), mechanism
# GEMINI_CLI_HOME
# (https://geminicli.com/docs/cli/enterprise/).
#
# 8.1 is the question this probe settles: GEMINI_CLI_HOME is documented as naming a directory CONTAINING
# `.gemini`, not the configuration directory itself. If that holds, the delta shows `.gemini/` created
# INSIDE the target -- which is why the snapshot recurses, and why the adapter must not declare the nested
# path as one of its own.
. sandbox/probes/common.sh

probe_npm_install @google/gemini-cli
probe_version gemini
probe_help gemini
probe_strings gemini GEMINI_API_KEY GOOGLE_API_KEY GOOGLE_APPLICATION_CREDENTIALS GEMINI_CLI_HOME
# `mcp list` RATHER THAN THE DEFAULT `--version`, and here the reason is determinism rather than an empty
# delta: `--version` DOES initialise, but what it leaves behind is a pair of
# `projects.json.<uuid>.tmp` files -- an interrupted write whose names differ on every run, so the delta
# a transcript commits would never reproduce. `mcp list` exits 0, needs no credentials, and completes the
# write: the same measurement arrives as `projects.json` plus a stable history and tmp tree.
#
# GEMINI_CLI_HOME names a HOME and not a config directory: measured, the agent creates `.gemini` INSIDE
# the target. The delta therefore shows `<target>/.gemini/...`, which is the variable working, not a
# nested-path defect.
probe_behaviour gemini "$PROBE_AGENT_HOME/.gemini" env:GEMINI_CLI_HOME @none mcp list
