# Cursor Agent CLI probe: `sandbox/run.sh probe cursor`.
#
# The executable is `cursor-agent`, not `cursor`. Installed by vendor script
# (https://cursor.com/install, documented at https://cursor.com/docs/cli/installation); there is no npm
# package, so this probe refuses a version argument.
#
# The installer runs as the AGENT, so its binary lands in the agent's `$PROBE_AGENT_HOME/.local/bin`, and
# it is reachable at step 2 because `common.sh`'s PROBE_AGENT_PATH names that directory. NOT because of the
# Containerfile's `ENV PATH`: that one is the harness's and, as its own comment says, "deliberately does NOT
# name the agent's bin directories". So a "command not found" at step 2 means the install failed, not that
# the probe looked in the wrong place.
#
# Default `$PROBE_AGENT_HOME/.cursor` and mechanism CURSOR_CONFIG_DIR, both from
# https://cursor.com/docs/cli/reference/configuration. The docs also describe an XDG fallback
# (`$XDG_CONFIG_HOME/cursor`), so the container delta is the record of which of the two it actually used.
. sandbox/probes/common.sh

probe_script_install https://cursor.com/install bash
probe_version cursor-agent
probe_help cursor-agent
probe_strings cursor-agent CURSOR_API_KEY CURSOR_CONFIG_DIR XDG_CONFIG_HOME
probe_behaviour cursor-agent "$PROBE_AGENT_HOME/.cursor" env:CURSOR_CONFIG_DIR @none
