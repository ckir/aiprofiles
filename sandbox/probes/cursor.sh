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
# (`$XDG_CONFIG_HOME/cursor`), so BOTH are watched and the delta is the record of which one it used.
#
# WATCHING ONLY THE FIRST WOULD NOT RECORD "it used the other" -- it would record NOTHING. `probe_delta`
# calls an empty delta the launch having changed nothing, unambiguously, so an agent that wrote its whole
# profile to the unwatched path would come back reading as clean isolation. That is the same class as the
# defect fixed in `4115027`, where every probe watched the harness's home instead of the agent's.
#
# `$XDG_CONFIG_HOME` is RESOLVED here rather than named. Nothing in this repository sets it, and
# `probe_as_agent` passes the agent exactly HOME, PATH and NPM_CONFIG_PREFIX through `env` -- `sudo`'s
# `env_reset` drops the rest -- so it is unset for the measured party. Whether cursor-agent then falls back
# to a `.config/cursor` directory under the agent's home or goes straight to `.cursor` when the variable is
# unset is NOT settled from this repository -- the vendor docs can be read either way, and nobody has
# verified it against the binary. That is why both are watched: the delta records whichever one it does,
# regardless of which reading is right. The `.config/cursor` path is chosen here because it is the XDG
# spec's own default for an unset variable, not because cursor-agent's behaviour with the variable unset
# has been confirmed. Writing `$XDG_CONFIG_HOME/cursor` literally would expand to `/cursor` and watch a
# path that cannot exist, which is the empty-delta failure above wearing the costume of a fix.
#
# That default is spelled out in words rather than as the shell variable it comes from, because the
# harness-home check scans these files WHOLE -- comments included -- and an unqualified reference to the
# HOME variable anywhere in a probe script is exactly what it exists to refuse. It caught the first draft
# of this comment, and then the sentence that was explaining why.
. sandbox/probes/common.sh

# AFTER the source, never before: `$PROBE_AGENT_HOME` is defined in `common.sh`. Set above it this line
# expands to the empty string, watches `/.cursor` and `/.config/cursor`, finds both absent -- and hands
# back the empty delta this whole comment exists to prevent. The other four probes that build a `default`
# variable -- opencode, amp, aider and continue -- all set it below their source line too.
default="$PROBE_AGENT_HOME/.cursor $PROBE_AGENT_HOME/.config/cursor"

probe_script_install https://cursor.com/install bash
probe_version cursor-agent
probe_help cursor-agent
probe_strings cursor-agent CURSOR_API_KEY CURSOR_CONFIG_DIR XDG_CONFIG_HOME
# NO LAUNCH OVERRIDE, and unlike Continue's case this one is positive: `--version` was measured to
# initialise here, writing `cli-config.json` into the target with CURSOR_CONFIG_DIR set and nothing at
# either default location. The variable is honoured and the default launch already demonstrates it, so
# overriding it would add a heavier command for no extra evidence.
#
# The only default-location writes measured were a `cursor-compile-cache` under `.cache` and a `.running`
# marker inside the installation itself -- neither is configuration, and neither is watched.
probe_behaviour cursor-agent "$default" env:CURSOR_CONFIG_DIR @none
