# Pi probe: `sandbox/run.sh probe pi [version]`.
#
# Package @earendil-works/pi-coding-agent (registry-verified 2026-09-16: repository
# github.com/earendil-works/pi, `bin` is `pi`). This is the pi.dev coding agent, not one of the several
# unrelated tools named `pi`; the repository field is what distinguishes them.
#
# `--ignore-scripts` is part of the vendor's own documented npm command (https://pi.dev/), so it is passed
# rather than dropped -- an install run differently from the way the vendor documents it measures
# something the vendor does not ship.
#
# Default `$PROBE_AGENT_HOME/.pi/agent` and mechanism PI_CODING_AGENT_DIR are the LEAST well cited of
# the twelve: the path comes from the providers documentation's resolution order rather than a sentence
# stating the default. Step 6's delta is what settles it, and if the delta is empty the adapter is an
# outcome-2.
#
# THE WATCH IS THE PARENT, `.pi`, DELIBERATELY WIDER THAN THAT DEFAULT -- and the two differing is not a
# slip. `probe_snapshot` runs `find` with no `-maxdepth` on a directory, so watching `.pi` already carries
# `.pi/agent` as a subtree. Since the citation is the weakest of the twelve, watching only the cited child
# would turn "our path guess was wrong" into an empty delta, which `probe_delta` calls the launch having
# changed nothing, unambiguously. The wider watch is what lets a wrong guess show up as evidence.
. sandbox/probes/common.sh

probe_npm_install @earendil-works/pi-coding-agent --ignore-scripts
probe_version pi
probe_help pi
probe_strings pi ANTHROPIC_API_KEY OPENAI_API_KEY PI_CODING_AGENT_DIR
# `-p hello` RATHER THAN THE DEFAULT `--version`. Measured per launch: `--version`, `list` and
# `auth check` all write nothing, so the mechanism delta is empty whatever the variable does. `-p` is
# non-interactive mode, and it INITIALISES BEFORE IT FAILS -- with no API key it exits 1 saying so,
# having already written `auth.json`, `models-store.json` and a sessions directory.
#
# PROBE_EXPECT=1 IS WHAT MAKES THAT USABLE, and this file previously claimed it needed nothing. It said a
# non-zero exit was "the honest record" that "the transcript carries ... so a reader can see which it
# was" -- a property of the harness asserted without reading the harness, and the first real probe run
# refuted it: `probe_record` routes any unexpected non-zero code to `failures`, `probe_finish` derives
# the run's status from that file, the job failed, and `verify-transcripts.sh` accepts a transcript only
# when `probe-exit` is 0. The measurement had succeeded and the probe reported failure.
#
# Declaring the code keeps the step strict rather than excusing it: 1 is a measurement, and anything else
# -- including 0 -- still fails. If pi stops requiring a key, this step goes red, which is the right
# alarm, because the comment above would then be describing an agent that no longer behaves that way.
PROBE_EXPECT=1
#
# This also settles the default location, which the design listed as unverified: it is
# `$PROBE_AGENT_HOME/.pi`, with the files one level down under `agent/`. Under the variable the same
# files land directly in the target, without the `.pi/agent` prefix.
probe_behaviour pi "$PROBE_AGENT_HOME/.pi" env:PI_CODING_AGENT_DIR @none -p hello
