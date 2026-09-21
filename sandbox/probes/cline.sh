# Cline CLI probe: `sandbox/run.sh probe cline [version]`.
#
# Package cline (registry-verified 2026-09-16: repository github.com/cline/cline, `bin` is `cline` -- the
# name is generic enough to be worth checking rather than assuming). Default `$PROBE_AGENT_HOME/.cline`
# (https://docs.cline.bot/cli/cli-reference).
#
# 8.3 says the mechanism is undecided BY DESIGN and names the three questions this probe answers, in
# order: does CLINE_DATA_DIR isolate state; does it enable sandbox mode; does --config work alongside it.
# Hence three mechanism repetitions rather than one. `--config` and `--data-dir` both name DIRECTORIES
# here, not files, which is why they are `flagdir:` -- the docs describe --config as a "Configuration
# directory" and --data-dir as "isolated local state at this directory path".
#
# Two first-party sources disagree about --config's default by one path segment (docs.cline.bot says
# ~/.cline/data/settings, the repository README implies ~/.cline/data). The probe does not have to resolve
# that: it watches `$PROBE_AGENT_HOME/.cline`, which contains both.
. sandbox/probes/common.sh

probe_npm_install cline
probe_version cline
probe_help cline
probe_strings cline ANTHROPIC_API_KEY CLINE_API_KEY OPENAI_API_KEY OPENROUTER_API_KEY CLINE_DATA_DIR
# NO LAUNCH OVERRIDE, because this agent could not be measured at all on the machine that measured the
# other eleven. `cline --version` and `cline --help` both die with SIGILL -- "Illegal instruction (core
# dumped)", exit 132 -- before printing anything. The host was a Podman WSL virtual machine whose
# /proc/cpuinfo advertises `avx` and no `avx2`, and cline ships native code; the ordinary Linux CI runner
# does advertise avx2, so the expectation is that it runs there. Treat the launch commands below as
# UNMEASURED rather than chosen, and settle them from the Sandbox workflow's run. Do NOT record the crash
# as a property of the agent on this evidence: it is a property of that CPU and this agent together.
probe_behaviour cline "$PROBE_AGENT_HOME/.cline" env:CLINE_DATA_DIR @none
probe_behaviour cline "$PROBE_AGENT_HOME/.cline" flagdir:--data-dir @none
probe_behaviour cline "$PROBE_AGENT_HOME/.cline" flagdir:--config @none
