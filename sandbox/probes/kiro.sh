# Kiro CLI probe: `sandbox/run.sh probe kiro`.
#
# Kiro CLI is the rebrand of Amazon Q Developer CLI and the executable is `kiro-cli`, not `kiro` -- the id
# names the file, the executable names the thing launched (7.3).
#
# Installed by vendor script (https://cli.kiro.dev/install, documented at https://kiro.dev/cli/). There is
# no npm package, so this probe takes NO version argument and refuses one rather than recording a version
# the installer never saw. The `.deb` 8's table also lists is documented for the Kiro IDE, not for
# kiro-cli, so it is not used here.
#
# 8.5 is what this probe tests: KIRO_HOME is documented, but an open upstream report describes
# subsystems ignoring it and using ~/.kiro regardless, failing silently. That shape shows up as a delta at
# BOTH locations -- files under the target AND new files under $PROBE_AGENT_HOME/.kiro -- which is
# exactly what `NotGuaranteed` exists to describe. A probe that watched only the target would have
# called it isolated.
. sandbox/probes/common.sh

# TWO locations. `~/.kiro` is the documented one and it is correct -- MEASURED with no KIRO_HOME set, the
# settings land at `$PROBE_AGENT_HOME/.kiro/settings/cli.json`. But the agent writes a SECOND tree the
# documentation does not mention, `$PROBE_AGENT_HOME/.local/share/kiro-cli`, holding `data.sqlite3`, a
# run-receipts directory and a telemetry lock -- and KIRO_HOME does not move any of it. That split is
# exactly the partial-isolation shape 8.5 predicts, now measured rather than inferred from a bug report:
# settings follow the variable, data does not. Watching only `~/.kiro` would have recorded the settings
# moving and called it isolated.
default="$PROBE_AGENT_HOME/.kiro $PROBE_AGENT_HOME/.local/share/kiro-cli"

probe_script_install https://cli.kiro.dev/install bash
probe_version kiro-cli
probe_help kiro-cli
probe_strings kiro-cli KIRO_API_KEY KIRO_HOME AWS_ACCESS_KEY_ID AWS_PROFILE
# `settings all` RATHER THAN THE DEFAULT `--version`, which was measured to write nothing at all at either
# location -- an empty delta that says nothing about KIRO_HOME. `settings all` exits 0, prints nothing,
# needs no credentials, and writes the settings file the mechanism is supposed to move. `doctor` also
# writes but exits 1 when unauthenticated.
probe_behaviour kiro-cli "$default" env:KIRO_HOME @none settings all
