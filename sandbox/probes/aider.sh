# Aider probe: `sandbox/run.sh probe aider [version]`.
#
# A re-probe; see claude.sh for why it resolves latest rather than pinning `aider.rs:32`'s 0.86.2.
# Python 3.12 is pinned because newer interpreters can start long source builds of Aider's native
# dependencies -- that is the INTERPRETER pin CONTRIBUTING.md:108-109 describes, not a package pin.
#
# Aider's default location is a FILE, `$HOME/.aider.conf.yml` (`aider.rs:15`), not a directory.
#
# The candidate sweep re-establishes SP2 design D5's finding -- missing, empty and comment-only each exit
# 2 while `{}` is accepted (`aider.rs:17-18`). It was measured once and never recorded; 7.4 says a
# transcript is what makes a measurement evidence, so the one adapter whose file content is already known
# is also the one that proves the sweep reports what SP2 found.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=.aider.conf.yml

probe_uv_install aider-chat 3.12
probe_version aider
probe_help aider
probe_strings aider OPENAI_API_KEY ANTHROPIC_API_KEY AIDER_MODEL
probe_candidates aider flagfile:--config @none "" "# nothing" "{}"
probe_behaviour aider "$HOME/.aider.conf.yml" flagfile:--config "{}"
