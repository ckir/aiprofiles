# OpenCode probe: `sandbox/run.sh probe opencode [version]`.
#
# Package opencode-ai (registry-verified 2026-09-16; `bin` is `opencode`). The vendor documents a curl
# script first and npm second (https://opencode.ai/docs); npm is used here because it is the one that can
# honour a pinned version, which 7.4 needs to repeat a measurement.
#
# TWO default locations, and that is 8.2's whole point: config lives at
# `$PROBE_AGENT_HOME/.config/opencode` while `auth.json` lives at
# `$PROBE_AGENT_HOME/.local/share/opencode` (https://opencode.ai/docs/providers/), which
# OPENCODE_CONFIG_DIR does not move. Watching only the first would record "credentials did not appear in
# the target" -- an absence of evidence -- where watching both records WHERE they went, which 6 says is
# enough to claim NotSupported honestly rather than Unknown.
#
# Two mechanisms, because 7.3's Q2 asks which of them the adapter should use: OPENCODE_CONFIG_DIR names
# a directory and OPENCODE_CONFIG names a single file (https://opencode.ai/docs/cli/).
. sandbox/probes/common.sh

default="$PROBE_AGENT_HOME/.config/opencode $PROBE_AGENT_HOME/.local/share/opencode"

probe_npm_install opencode-ai
probe_version opencode
probe_help opencode
probe_strings opencode ANTHROPIC_API_KEY OPENAI_API_KEY OPENCODE_CONFIG_DIR OPENCODE_CONFIG XDG_DATA_HOME
probe_behaviour opencode "$default" env:OPENCODE_CONFIG_DIR @none
PROBE_CONFIG_NAME=opencode.json
probe_behaviour opencode "$default" envfile:OPENCODE_CONFIG "{}"
