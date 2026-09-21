# Amp probe: `sandbox/run.sh probe amp [version]`.
#
# Package @ampcode/cli, NOT @sourcegraph/amp. Both resolve and carry the same version, which looks like a
# choice; it is not. @sourcegraph/amp is a rename stub whose only dependency is @ampcode/cli and whose own
# registry description reads "Renamed to @ampcode/cli" (registry-verified 2026-09-16). Installing the stub
# would work and would record the wrong package as the thing measured.
#
# Default `$PROBE_AGENT_HOME/.config/amp/settings.json` and mechanism `--settings-file <path>`, both from
# https://ampcode.com/docs/cli/settings
#
# The same page documents that workspace settings override user settings and managed settings override
# both. That is a layering the flag cannot defeat, so it belongs in the adapter's basis the way Aider's
# layering note does (`aider.rs`'s `LAYERING_NOTE` constant) -- the probe records `--help` and the container
# delta, and SP4b writes the claim.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=settings.json
# TWO locations, and the second is the one that carries the measurement. The documented settings file
# lives under `.config/amp`, but MEASURED in a container, `.config/amp` is never written at all by any
# command this probe can run without credentials -- what the agent actually writes is
# `$PROBE_AGENT_HOME/.local/share/amp/device-id.json`, and `--settings-file` does NOT move it. Watching
# only the documented path recorded an empty delta and would have read as "the flag relocates
# everything", which is the opposite of what happens. Same shape as opencode's state directory: the
# documented location and the written location are not the same set.
default="$PROBE_AGENT_HOME/.config/amp $PROBE_AGENT_HOME/.local/share/amp"

probe_npm_install @ampcode/cli
probe_version amp
probe_help amp
probe_strings amp AMP_API_KEY AMP_URL AMP_SETTINGS_FILE
# See aider.sh: recorded before the first launch of either step, so both stay attributable.
# shellcheck disable=SC2086 # <default-location> is a whitespace-separated LIST; the split is the point
probe_pristine $default
# `logout` RATHER THAN THE DEFAULT `--version`. Measured per launch in a container: `--version` writes
# nothing anywhere, so the mechanism delta is empty whatever the flag does. `logout` exits 0 on a machine
# that was never logged in ("Already logged out."), needs no credentials, and is the cheapest command
# found that makes the agent write -- it creates `device-id.json`. `threads list` writes the same file but
# exits 1 without an API key, which would put a row in `failures` for a measurement that succeeded.
probe_behaviour amp "$default" flagfile:--settings-file "{}" logout
probe_candidates amp flagfile:--settings-file @none "" "{}"
