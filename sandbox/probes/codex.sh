# Codex CLI probe: `sandbox/run.sh probe codex`.
. sandbox/probes/common.sh

probe_record install npm install --global @openai/codex
probe_record version codex --version
probe_record help codex --help
probe_agent_profile codex
