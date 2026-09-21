# Continue CLI probe: `sandbox/run.sh probe continue [version]`.
#
# Package @continuedev/cli (registry-verified 2026-09-16; `bin` is `cn`, which is why the executable and
# the id differ). The vendor documents a shell script first and npm second; npm is used because it can
# honour a pinned version. Default `$PROBE_AGENT_HOME/.continue`, holding `config.yaml`.
#
# THE FLAG NAME IS NOT ESTABLISHED. 8's table claims `--config <file>`, and a documentation pass on
# 2026-09-16 could not find a first-party page showing that flag for `cn`. Step 3's `--help` capture is
# the oracle: if the flag is spelled differently, the behaviour step records a non-zero exit and the
# transcript shows the real spelling one file away. That is a 9 outcome, not a probe failure -- and it is
# why SP4b plans the adapter only after the transcripts exist.
#
# The candidate sweep answers 8.4 for this agent: `agent-profile` must create the file before `cn` reads
# it, so the smallest content that sets no option is part of the mechanism.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=config.yaml
default="$PROBE_AGENT_HOME/.continue"

probe_npm_install @continuedev/cli
probe_version cn
probe_help cn
probe_strings cn CONTINUE_API_KEY ANTHROPIC_API_KEY OPENAI_API_KEY
# See aider.sh: recorded before the first launch of either step, so both stay attributable.
probe_pristine "$default"
# NO LAUNCH OVERRIDE, and that is a measured result rather than an omission. `--version` and `ls` were
# both measured in a container and neither writes anything at either location; Continue appears to
# initialise nothing before it has credentials or a session. So there is no cheap command that would make
# this delta informative, and the probe records the empty delta as the fact it is rather than inventing a
# launch that looks better. Do not "fix" this by adding one without measuring it first.
#
# The flag name IS confirmed: `cn --help` lists `--config <path>`, which the design had carried as
# unverified.
probe_behaviour cn "$default" flagfile:--config "{}"
probe_candidates cn flagfile:--config @none "" "# nothing" "{}"
