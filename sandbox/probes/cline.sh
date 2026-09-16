# Cline CLI probe: `sandbox/run.sh probe cline [version]`.
#
# Package cline (registry-verified 2026-09-16: repository github.com/cline/cline, `bin` is `cline` -- the
# name is generic enough to be worth checking rather than assuming). Default `$HOME/.cline`
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
# that: it watches `$HOME/.cline`, which contains both.
. sandbox/probes/common.sh

probe_npm_install cline
probe_version cline
probe_help cline
probe_strings cline ANTHROPIC_API_KEY CLINE_API_KEY OPENAI_API_KEY OPENROUTER_API_KEY CLINE_DATA_DIR
probe_behaviour cline "$HOME/.cline" env:CLINE_DATA_DIR @none
probe_behaviour cline "$HOME/.cline" flagdir:--data-dir @none
probe_behaviour cline "$HOME/.cline" flagdir:--config @none
