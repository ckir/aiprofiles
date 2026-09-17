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
probe_behaviour gemini "$PROBE_AGENT_HOME/.gemini" env:GEMINI_CLI_HOME @none
