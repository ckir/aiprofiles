# Codex CLI probe: `sandbox/run.sh probe codex [version]`.
#
# A re-probe; see claude.sh for why it resolves latest rather than pinning `codex.rs:27`'s 0.153.4.
# Package @openai/codex, executable `codex`, mechanism CODEX_HOME (`codex.rs:13`).
. sandbox/probes/common.sh

probe_npm_install @openai/codex
probe_version codex
probe_help codex
probe_strings codex OPENAI_API_KEY OPENAI_BASE_URL CODEX_API_KEY
probe_behaviour codex "$HOME/.codex" env:CODEX_HOME @none
