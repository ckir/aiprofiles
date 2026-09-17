# Aider probe: `sandbox/run.sh probe aider [version]`.
#
# A re-probe; see claude.sh for why it resolves latest rather than pinning
# `aider.rs`'s `upstream_version` field (0.86.2).
# Python 3.12 is pinned because newer interpreters can start long source builds of Aider's native
# dependencies -- that is the INTERPRETER pin CONTRIBUTING.md's "Pin interpreter versions an agent
# supports" guidance describes, not a package pin.
#
# Aider's default location is a FILE, `$HOME/.aider.conf.yml` (`aider.rs`'s `FILE_NAME` constant), not a directory.
#
# The candidate sweep re-establishes SP2 design D5's finding -- missing, empty and comment-only each exit
# 2 while `{}` is accepted (`aider.rs`'s `INITIAL_CONFIG` constant). It was measured once and never
# recorded; 7.4 says a transcript is what makes a measurement evidence, so the one adapter whose file
# content is already known is also the one that proves the sweep reports what SP2 found.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=.aider.conf.yml
default="$HOME/.aider.conf.yml"

probe_uv_install aider-chat 3.12
probe_version aider
probe_help aider
probe_strings aider OPENAI_API_KEY ANTHROPIC_API_KEY AIDER_MODEL
# The default location's pre-launch state, recorded before the first launch of any kind. `probe_behaviour`
# would register it lazily, but this script launches the agent for the acceptance sweep as well, and only
# a state recorded before BOTH steps keeps both of them attributable whatever order they end up in.
probe_pristine "$default"
# Behaviour BEFORE the sweep: §7.3's order. Each sweep launch re-initialises the agent's default location,
# and a baseline taken after them attributes their files to the install (`common.sh`, "keeping every
# launch attributable").
probe_behaviour aider "$default" flagfile:--config "{}"
probe_candidates aider flagfile:--config @none "" "# nothing" "{}"
