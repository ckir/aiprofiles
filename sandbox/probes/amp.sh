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
default="$PROBE_AGENT_HOME/.config/amp"

probe_npm_install @ampcode/cli
probe_version amp
probe_help amp
probe_strings amp AMP_API_KEY AMP_URL AMP_SETTINGS_FILE
# See aider.sh: recorded before the first launch of either step, so both stay attributable.
probe_pristine "$default"
probe_behaviour amp "$default" flagfile:--settings-file "{}"
probe_candidates amp flagfile:--settings-file @none "" "{}"
