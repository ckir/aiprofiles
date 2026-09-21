# OpenCode probe: `sandbox/run.sh probe opencode [version]`.
#
# PACKAGE `@opencode/cli`, NOT `opencode-ai`. The vendor publishes two live lines under two different
# names, and picking the wrong one measures the wrong agent: `opencode-ai` is the 1.x line (1.18.31) and
# `@opencode/cli` is the 2.x line (2.0.12). Neither is deprecated and BOTH were published on 2026-09-21,
# so "the newer name superseded the older" is not the relationship -- they ship in parallel. 2.x is the
# line measured here by owner decision; 1.x is Linux-only and is planned as a second probe rather than a
# replacement for this one, which is why nothing below is written as if 1.x had gone away.
#
# THREE default locations, where the design note said two. `OPENCODE_CONFIG_DIR` moves the config
# directory and nothing else, so config, data and state have to be watched separately to say WHERE each
# one went (https://opencode.ai/docs/providers/). The third is the one a reader will not predict:
# `$PROBE_AGENT_HOME/.local/state/opencode` holds `service.json`, which 2.x writes on every launch that
# needs its background server. Watching only config and data would have recorded "state did not move" as
# an absence of evidence -- precisely the failure the two-location note was written to avoid, one
# location further out. `$PROBE_AGENT_HOME/.cache/opencode` is deliberately NOT watched: it holds a
# downloaded binary, not configuration or credentials.
#
# Two mechanisms, because the design's Q2 asks which of them the adapter should use: OPENCODE_CONFIG_DIR
# names a directory and OPENCODE_CONFIG names a single file (https://opencode.ai/docs/cli/). Both names
# are present in the 2.x binary, so the question is live for this line and not inherited from 1.x.
. sandbox/probes/common.sh

default="$PROBE_AGENT_HOME/.config/opencode $PROBE_AGENT_HOME/.local/share/opencode"
default="$default $PROBE_AGENT_HOME/.local/state/opencode"

probe_npm_install @opencode/cli
probe_version opencode2
probe_help opencode2
probe_strings opencode2 ANTHROPIC_API_KEY OPENAI_API_KEY OPENCODE_CONFIG_DIR OPENCODE_CONFIG XDG_DATA_HOME

# `models` RATHER THAN THE DEFAULT `--version`, and this is the difference between a measurement and an
# empty box. Measured in a container, per launch, counting what appeared: `--version` and `--help` both
# exit 0 and create the default directories, but write no configuration FILE into any of them. A
# mechanism test launched that way therefore reports an empty target delta whatever the mechanism does,
# which reads as "the variable moved nothing" when it means "nothing was asked to move". `models` is the
# cheapest command found that writes one, and it needs no credentials to do it: it exits 0 with an empty
# stdout on a machine with no provider configured, and still writes `service.json`.
probe_behaviour opencode2 "$default" env:OPENCODE_CONFIG_DIR @none models

# EVERY LAUNCH THAT STARTS THE BACKGROUND SERVER LEAVES IT RUNNING, and a surviving server is the one
# piece of state this harness cannot restore. `probe_behaviour` re-baselines by putting FILES back; it
# has no concept of a process, so without this step the second measurement below attaches to the server
# the first one started instead of starting its own -- and that server is still holding the first
# mechanism's configuration. The two measurements would then not be independent, which is the whole
# reason each one re-baselines. The stop runs AFTER the snapshot `probe_behaviour` has already taken, so
# it cannot alter what was recorded, and it carries the same environment as the launch because each
# configuration directory has its own server. `service stop` exits 0 when no server is running, so this
# is an ordinary recorded step and not a soft one.
#
# The re-measurement that proves the concern is real also shows it did not change THIS pair's result:
# killing the server between the two launches reproduced both deltas unchanged. The step is here so the
# probe is correct by construction rather than by luck, since a vendor that changes when the server is
# reused would otherwise silently corrupt the second measurement.
probe_record_agent service-stop-dir env "OPENCODE_CONFIG_DIR=$PROBE_TARGET" opencode2 service stop

PROBE_CONFIG_NAME=opencode.json
probe_behaviour opencode2 "$default" envfile:OPENCODE_CONFIG "{}" models

# No environment override here on purpose: OPENCODE_CONFIG names a file to READ and does not relocate
# what the agent WRITES, so the launch above ran against the default configuration directory and that is
# the server this stops.
probe_record_agent service-stop-file opencode2 service stop
