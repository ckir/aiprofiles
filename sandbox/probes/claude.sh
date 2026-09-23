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
# `mcp list` RATHER THAN THE DEFAULT `--version`, and the default is why this file was measured at all.
# PR #28 chose a launch command per agent because `--version` writes nothing for most of them and an
# empty delta proves nothing about the mechanism; that sweep covered the nine SP4b agents and never
# touched the three SP2 re-probes, so this one kept the default. Probe run 35845001948 shows the cost:
# claude's delta was empty while codex's -- whose `--version` happens to write -- was not.
#
# MEASURED in this image, agent uid, home warmed with `--version` then `--help` first and only the
# watched locations reset between candidates, so that first-run initialisation is not credited to
# whichever command is being tested (the trap `copilot.sh` records):
#
#     --version   exit 0   nothing
#     --help      exit 0   nothing
#     config ls   exit 1   .claude.json, projects/, sessions/, backups/
#     mcp list    exit 0   .claude.json, backups/
#     doctor      exit 0   .claude.json, backups/
#     -p hi       exit 1   .claude.json, projects/, sessions/, backups/
#
# `mcp list` and `doctor` are equivalent in what they write; `mcp list` is the one whose job is plainly to
# READ configuration, so it is the smaller claim on the agent's behaviour. `config ls` and `-p hi` write
# more, and both exit 1 -- a row in `failures` for a measurement that succeeded, the same trade amp.sh
# refuses. `.claude.json` is the file an adapter parses, and it lands inside CLAUDE_CONFIG_DIR.
probe_behaviour claude "$PROBE_AGENT_HOME/.claude" env:CLAUDE_CONFIG_DIR @none mcp list
