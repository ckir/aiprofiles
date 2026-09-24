# Aider probe: `sandbox/run.sh probe aider [version]`.
#
# A re-probe; see claude.sh for why it resolves latest rather than pinning
# `aider.rs`'s `upstream_version` field (0.86.2).
# Python 3.12 is pinned because newer interpreters can start long source builds of Aider's native
# dependencies -- that is the INTERPRETER pin CONTRIBUTING.md's "Pin interpreter versions an agent
# supports" guidance describes, not a package pin.
#
# Aider's default location is a FILE, `$PROBE_AGENT_HOME/.aider.conf.yml` (`aider.rs`'s `FILE_NAME`
# constant), not a directory.
#
# The candidate sweep re-establishes SP2 design D5's finding -- missing, empty and comment-only each exit
# 2 while `{}` is accepted (`aider.rs`'s `INITIAL_CONFIG` constant). It was measured once and never
# recorded; 7.4 says a transcript is what makes a measurement evidence, so the one adapter whose file
# content is already known is also the one that proves the sweep reports what SP2 found.
. sandbox/probes/common.sh

PROBE_CONFIG_NAME=.aider.conf.yml
# TWO locations. The documented one is the config file the flag names; the second is where aider puts
# STATE, and `--config` does not move it. MEASURED in this image: `--just-check-update` creates
# `~/.aider/analytics.json` and `~/.aider/caches/versioncheck` while the flag points elsewhere, so the
# leak is the finding and watching only the config file would have recorded an empty delta and read as
# "the flag relocates everything". Same shape as amp and opencode: the documented location and the
# written location are not the same set.
default="$PROBE_AGENT_HOME/.aider.conf.yml $PROBE_AGENT_HOME/.aider"

probe_uv_install aider-chat 3.12
probe_version aider
probe_help aider
probe_strings aider OPENAI_API_KEY ANTHROPIC_API_KEY AIDER_MODEL
# The default location's pre-launch state, recorded before the first launch of any kind. `probe_behaviour`
# would register it lazily, but this script launches the agent for the acceptance sweep as well, and only
# a state recorded before BOTH steps keeps both of them attributable whatever order they end up in.
# shellcheck disable=SC2086 # <default-location> is a whitespace-separated LIST; the split is the point
probe_pristine $default
# Behaviour BEFORE the sweep: §7.3's order. Each sweep launch re-initialises the agent's default location,
# and a baseline taken after them attributes their files to the install (`common.sh`, "keeping every
# launch attributable").
# `--just-check-update` RATHER THAN THE DEFAULT `--version`. PR #28 measured a launch command per agent
# and never covered the three SP2 re-probes, so this one kept a default that writes nothing anywhere.
# MEASURED here, home warmed first and only the watched locations reset between candidates:
#
#     --version            exit 0     nothing
#     --help               exit 0     nothing
#     --just-check-update  exit 0     ~/.aider/{analytics.json,caches/versioncheck}
#     --exit --yes         exit 124   KILLED at 60s
#     --message hi --exit  exit 124   KILLED at 60s
#
# The two that would drive a chat HANG and are killed by the timeout, which is why the cheap
# update check is the one that can run in CI. What it proves is NEGATIVE and that is the point: the
# flag names a config file the agent READS, so the acceptance sweep below is what shows config is
# honoured, while this step shows state landing in the HOME regardless of the flag.
#
# AIDER ALSO WRITES `.aider.chat.history.md` INTO THE WORKING DIRECTORY -- measured, in a writable cwd --
# which is what `aider.rs`'s StateIsolation basis cites. The probe cwd (`/home/probe/work`, setgid
# `probe-share`) IS agent-writable, so such a write is captured by the whole-container diff; it is not in
# this delta, because the cwd is not a watched location and holds the extracted repository.
probe_behaviour aider "$default" flagfile:--config "{}" --just-check-update
probe_candidates aider flagfile:--config @none "" "# nothing" "{}"
