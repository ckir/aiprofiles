# Claude Code probe: `sandbox/run.sh probe claude [version]`.
#
# A re-probe. SP2 measured 2.1.270 and never committed a transcript (7.4), so there is nothing to
# reproduce and this resolves latest rather than pinning -- the fold commit updates `claude.rs`'s
# `upstream_version` field to whatever it finds.
#
# Package @anthropic-ai/claude-code, executable `claude`. The documented bypass list is longer than any
# one page: `claude.rs`'s evidence `notes` field records ANTHROPIC_PROFILE and the BEDROCK/VERTEX switches
# as "binary strings measured", so step 4 is the step that established them and this re-probe has to
# reproduce it.
. sandbox/probes/common.sh

probe_npm_install @anthropic-ai/claude-code
probe_version claude
probe_help claude
probe_strings claude \
    ANTHROPIC_API_KEY ANTHROPIC_AUTH_TOKEN ANTHROPIC_PROFILE \
    CLAUDE_CODE_OAUTH_TOKEN CLAUDE_CODE_OAUTH_REFRESH_TOKEN \
    CLAUDE_CODE_USE_BEDROCK CLAUDE_CODE_USE_VERTEX CLAUDE_CODE_USE_FOUNDRY
probe_behaviour claude "$PROBE_AGENT_HOME/.claude" env:CLAUDE_CONFIG_DIR @none
