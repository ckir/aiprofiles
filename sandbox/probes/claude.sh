# Claude Code probe: `sandbox/run.sh probe claude`.
. sandbox/probes/common.sh

probe_record install npm install --global @anthropic-ai/claude-code
probe_record version claude --version
probe_record help claude --help
probe_agent_profile claude
