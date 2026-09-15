# Aider probe: `sandbox/run.sh probe aider`. Python 3.12 is pinned: newer interpreters can start long
# source builds of Aider's native dependencies.
. sandbox/probes/common.sh

probe_record install uv tool install --python 3.12 aider-chat
probe_record version aider --version
probe_record help aider --help
probe_agent_profile aider
