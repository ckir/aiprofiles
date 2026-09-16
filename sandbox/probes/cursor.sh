# Cursor Agent CLI probe: `sandbox/run.sh probe cursor`.
#
# The executable is `cursor-agent`, not `cursor`. Installed by vendor script
# (https://cursor.com/install, documented at https://cursor.com/docs/cli/installation); there is no npm
# package, so this probe refuses a version argument.
#
# The installer puts its binary in `$HOME/.local/bin`, which `sandbox/Containerfile:23` already has on
# PATH -- so a "command not found" at step 2 means the install failed, not that the probe looked in the
# wrong place.
#
# Default `$HOME/.cursor` and mechanism CURSOR_CONFIG_DIR, both from
# https://cursor.com/docs/cli/reference/configuration. The docs also describe an XDG fallback
# (`$XDG_CONFIG_HOME/cursor`), so the container delta is the record of which of the two it actually used.
. sandbox/probes/common.sh

probe_script_install https://cursor.com/install bash
probe_version cursor-agent
probe_help cursor-agent
probe_strings cursor-agent CURSOR_API_KEY CURSOR_CONFIG_DIR XDG_CONFIG_HOME
probe_behaviour cursor-agent "$HOME/.cursor" env:CURSOR_CONFIG_DIR @none
