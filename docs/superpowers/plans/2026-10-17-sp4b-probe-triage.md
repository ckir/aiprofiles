# SP4b probe triage ledger

Run: 39024332187 — <https://github.com/ckir/aiprofiles/actions/runs/39024332187>
Triage date: 2026-10-17. All nine transcripts carry `custody: ci` and
`harness-commit: 16737cb79a0cde5bb698fae7784e69b9eeb6b551` (identical across all nine).

**Artifact retention:** the run's artifacts are kept for **90 days** — Tasks 3–11 must land
inside that window (i.e. by 2027-01-15) or the probe must be re-run to regenerate the
transcripts.

**Method note (§6):** every claim below quotes a line from a transcript
(`docs/evidence/<id>-<version>.md` candidates, held uncommitted) or from
`sandbox/probes/<id>.sh`. Where the run does not settle something, the question is marked
**unanswered** — design §13 forbids reasoning an answer into existence. Outcomes are the five
of design §9; each probe's `probe-exit:` is 0 (quoted per adapter below), so no outcome-2
signal arises from exit codes. All `exit-codes:` lines and deltas quoted verbatim.

## Evidence file names (for Tasks 3–11)

`<version>` = the transcript's `version-extracted` line, copied exactly:

| id | transcript in artifact | `version-extracted:` | `docs/evidence/` name |
|---|---|---|---|
| gemini | `sandbox-transcript-gemini/gemini-0.64.0.md` | `0.64.0` | `gemini-0.64.0.md` |
| copilot | `sandbox-transcript-copilot/copilot-1.0.98.md` | `1.0.98` | `copilot-1.0.98.md` |
| opencode | `sandbox-transcript-opencode/opencode-v2.0.32.md` | `v2.0.32` | `opencode-v2.0.32.md` |
| cline | `sandbox-transcript-cline/cline-3.0.73.md` | `3.0.73` | `cline-3.0.73.md` |
| pi | `sandbox-transcript-pi/pi-1.3.0.md` | `1.3.0` | `pi-1.3.0.md` |
| kiro | `sandbox-transcript-kiro/kiro-2.32.0.md` | `2.32.0` | `kiro-2.32.0.md` |
| cursor | `sandbox-transcript-cursor/cursor-2026.10.16-e3cb207.md` | `2026.10.16-e3cb207` | `cursor-2026.10.16-e3cb207.md` |
| continue | `sandbox-transcript-continue/continue-1.5.47.md` | `1.5.47` | `continue-1.5.47.md` |
| amp | `sandbox-transcript-amp/amp-0.0.1792224035-gabd4f6.md` | `0.0.1792224035-gabd4f6` | `amp-0.0.1792224035-gabd4f6.md` |

Note the `v` prefix survives for opencode (`opencode v2.0.32` → `v2.0.32`): the first
`[A-Za-z0-9._-]+` run containing a digit is `v2.0.32` itself (§7.3 step 2).

---

## gemini — Gemini CLI

- **Outcome: 1 — mechanism confirmed.** Decided by:
  `delta-env-GEMINI_CLI_HOME: + f /home/agent/probe-target/.gemini/history/work/.project_root`
  — the default location `/home/agent/.gemini` was created only
  `projects.json.<uuid>.tmp` leftovers during the baseline (§8.1's nesting, measured).
- **version-extracted:** `0.64.0` — transcript [gemini-0.64.0.md]
- **Mechanism evidence** (`probe_behaviour gemini "$PROBE_AGENT_HOME/.gemini" env:GEMINI_CLI_HOME @none mcp list`, `sandbox/probes/gemini.sh`):
  strings: `documented GEMINI_CLI_HOME: present`; also
  `documented GEMINI_API_KEY: present`, `documented GOOGLE_API_KEY: present`,
  `documented GOOGLE_APPLICATION_CREDENTIALS: present`. `help:` shows no env-var section —
  the token is not in the help capture (see the concern section at the end of this file).
- **Behaviour delta** (baseline target = `d /home/agent/probe-target`; default
  `d /home/agent/.gemini` with four `projects.json.<uuid>.tmp` files):
  at the target, the `.gemini/` tree *inside* it — `+ d /home/agent/probe-target/.gemini`,
  `+ f /home/agent/probe-target/.gemini/projects.json`, `+ f .../.gemini/history/work/.project_root`,
  `+ f .../.gemini/tmp/work/.project_root`; nothing appeared at the default location beyond
  the earlier baseline. So the variable moved config **and** history/state to the target,
  nested one level inside it — §8.1's claimed nesting holds exactly.
- **Capability state set (recommended):**
  - ConfigIsolation: **Supported** — basis quoting `+ d /home/agent/probe-target/.gemini` and
    `+ f /home/agent/probe-target/.gemini/projects.json`.
  - StateIsolation: **Supported** — basis quoting
    `+ d /home/agent/probe-target/.gemini/history` and `+ d .../.gemini/tmp`.
  - CredentialIsolation: **Unknown** — `unmeasured: naming the vendor session required`; strings
    confirm `GEMINI_API_KEY`, `GOOGLE_API_KEY`, `GOOGLE_APPLICATION_CREDENTIALS` are the bypass
    variables, but no stored-credential observation was made.
  - One `Unknown` ⇒ `Proven` permitted under D6/§4.1.

## copilot — GitHub Copilot CLI

- **Outcome: 5 — mechanism present but ineffective (delta empty).** Decided by:
  `delta-env-COPILOT_HOME: (no change: nothing was created, moved or removed at the watched
  locations)` while the token is in the artefacts:
  `documented COPILOT_HOME: present`, and help documents
  `--log-dir <directory>  Set log file directory (default: ~/.copilot/logs/)`.
- **version-extracted:** `1.0.98` — transcript [copilot-1.0.98.md]
- **Mechanism evidence:** the invocation is
  `probe_behaviour copilot "$PROBE_AGENT_HOME/.copilot" env:COPILOT_HOME @none`
  (`sandbox/probes/copilot.sh`); strings say `documented COPILOT_HOME: present`,
  `documented COPILOT_GITHUB_TOKEN: ABSENT` (see §8 corrections),
  `documented GH_TOKEN: present`, `documented GITHUB_TOKEN: present`. The help capture is
  truncated before its option list and does not name `COPILOT_HOME` there; it does cite
  `--log-dir <directory>  Set log file directory (default: ~/.copilot/logs/)` and
  `COPILOT_ALLOW_ALL` via `[env: COPILOT_ALLOW_ALL=]`.
- **Behaviour delta:** baseline: target exists, `/home/agent/.copilot` **(absent)**; delta:
  **(no change at either location)** — the launch wrote nothing before it needed the network.
- **Capability state set (recommended):**
  - ConfigIsolation: **Unknown** — basis quoting the delta ``(no change)`` line and the
    `(absent)` default: the variable may work for writes the agent never performed before
    needing credentials, so the delta was simply empty (§9 outcome 5, Unknown branch).
  - StateIsolation: **Unknown** — same delta.
  - CredentialIsolation: **Unknown** — `unmeasured:`; credential storage not located.
  - Three `Unknown`s ⇒ **Experimental** under Gate C/D6.

## opencode — OpenCode

- **Outcome: 1 — mechanism confirmed.** Decided by:
  `delta-env-OPENCODE_CONFIG_DIR: + f /home/agent/probe-target/service.json`
  plus `+ f /home/agent/.local/state/opencode/service.json` — config placement followed
  `OPENCODE_CONFIG_DIR` (writes that belong to the config dir landed in the target); the
  data tree under `.local/share/opencode` did not move, as §8.2 predicts.
- **version-extracted:** `v2.0.32` — transcript [opencode-v2.0.32.md]; package measured is
  `@opencode/cli` (strings: `# /home/agent/.local/bin/opencode2 -> .../@opencode/cli/bin/opencode.exe`),
  the 2.x line (§8 table's install column says "npm or install script" without the package name).
- **Mechanism evidence:** strings: `documented OPENCODE_CONFIG_DIR: present`,
  `documented OPENCODE_CONFIG: present`, `documented XDG_DATA_HOME: present`,
  `documented ANTHROPIC_API_KEY: present`, `documented OPENAI_API_KEY: present`.
- **Behaviour deltas, both repetitions (§7.3 step 5/6 pair):**
  - **`env:OPENCODE_CONFIG_DIR`** (target = probe-target): target received
    `+ f /home/agent/probe-target/service.json`; at defaults,
    `+ f /home/agent/.local/share/opencode/opencode.db(-shm/-wal)`,
    `+ d .../shell/...`, and `+ f /home/agent/.local/state/opencode/service.json` — config-file-backed
    state followed the config dir, data/state did not. (Both `service.json` copies appearing is the
    agent's own behaviour; the transcript is the record.)
  - **`envfile:OPENCODE_CONFIG`** with content `{}`: exit 0; accepted — the file is written to
    the target (`PROBE_CONFIG_NAME=opencode.json`) and read (`+ f
    /home/agent/.config/opencode/service.json`, `.local/state/opencode/service.json` also present);
    the data writes again stayed at `.local/share/opencode`. So `OPENCODE_CONFIG` selects a config
    **file to read** and relocates none of the agent's writes — the probe script's comment already
    asserts this, and the delta confirms the writes stayed default.
- **Capability state set (recommended):**
  - ConfigIsolation: **Supported** — basis quoting `+ f /home/agent/probe-target/service.json`
    (mechanism moved the config-side write into the target) and the second delta showing the
    default-config-dir placement when only `OPENCODE_CONFIG` is set.
  - StateIsolation: **NotSupported** — basis quoting
    `+ f /home/agent/.local/share/opencode/opencode.db` and `opencode.log` staying at
    `.local/share/opencode`: neither mechanism names a data directory and none exists — the
    strings scan contains no `OPENCODE_DATA*` / `OPENCODE_HOME` token, answering Q2.
  - CredentialIsolation: **Unknown** — `unmeasured:`; `auth.json` was never observed anywhere in
    the deltas (unauthenticated launches never write it), so §6's "probe establishes where
    credentials are stored" exception is not met.
  - One `Unknown` ⇒ `Proven` permitted, and §8.2's "splits config from credentials" holds.

## cline — Cline CLI

- **Outcome: 5 — mechanism present but ineffective (delta empty at all three repetitions).**
  Decided by: `delta-env-CLINE_DATA_DIR: (no change: nothing was created, moved or removed at
  the watched locations)` (identical for `delta-flagdir---data-dir` and `delta-flagdir---config`),
  while the tokens are in the artefacts: `documented CLINE_DATA_DIR: present` and help lists
  `--config <path>  Configuration directory (default: ~/.cline)` and
  `--data-dir <path>  Use isolated local state at this directory path (default: ~/.cline/data)`.
- **version-extracted:** `3.0.73` — transcript [cline-3.0.73.md]
- **Mechanism evidence:** the help capture above; strings: `documented ANTHROPIC_API_KEY:
  present`, `documented CLINE_API_KEY: present`, `documented OPENAI_API_KEY: present`,
  `documented OPENROUTER_API_KEY: present`. (`probe_strings cline ANTHROPIC_API_KEY CLINE_API_KEY
  OPENAI_API_KEY OPENROUTER_API_KEY CLINE_DATA_DIR`, `sandbox/probes/cline.sh`)
- **Behaviour delta, all three mechanisms applied** (`env:CLINE_DATA_DIR`, `flagdir:--data-dir`,
  `flagdir:--config`, each re-baselined against the default `d /home/agent/.cline` with only
  `f /home/agent/.cline/cli-node-extra-ca-certs.pem` in it):
  **(no change: nothing was created, moved or removed at the watched locations)** at both the
  target and `~/.cline`. The agent wrote **nothing** under any of these mechanisms —
  the probe answered none of the three questions §8.3 poses from an empty delta. Note also that
  `cli-node-extra-ca-certs.pem` appeared at `~/.cline` *before* the launch — so the agent does
  write at `~/.cline` before any launch (the version/help steps).
- **Q1: unanswered.** The three deltas are empty; isolation and sandbox-mode side effects can
  be observed only from output files, and none were produced before the agent needed the network.
- **Capability state set (recommended):**
  - ConfigIsolation: **Unknown** — basis quoting the two delta lines above (the delta was
    empty; §9 outcome 5 Unknown branch).
  - StateIsolation: **Unknown** — same.
  - CredentialIsolation: **Unknown** — `unmeasured:`.
  - ⇒ **Experimental** under Gate C/D6.

## pi — Pi

- **Outcome: 1 — mechanism confirmed.** Decided by:
  `delta-env-PI_CODING_AGENT_DIR: + f /home/agent/probe-target/auth.json`,
  `+ f /home/agent/probe-target/models-store.json`, `+ d /home/agent/probe-target/sessions`
  — the baseline already held a `.pi/agent/auth.json` and `models-store.json` at the default,
  and the same files moved under the variable **without** a `.pi/agent` prefix, so
  `PI_CODING_AGENT_DIR` names the config directory itself. The behaviour step's exit code is `1`
  (`behaviour-env-PI_CODING_AGENT_DIR 1`) — that is the expected value: the probe script sets
  `PROBE_EXPECT=1` because with no API key `-p hello` exits 1 having already written its files
  (`sandbox/probes/pi.sh`), and `probe-exit:` is 0.
- **version-extracted:** `1.3.0` — transcript [pi-1.3.0.md]; help says
  `PI_CODING_AGENT_DIR  - Config directory (default: ~/.pi/agent)`.
- **Mechanism evidence:**
  `probe_strings pi ANTHROPIC_API_KEY OPENAI_API_KEY PI_CODING_AGENT_DIR`,
  the help capture contains `PI_CODING_AGENT_DIR`, `PI_CODING_AGENT_SESSION_DIR`,
  `PI_PACKAGE_DIR`, `PI_OFFLINE`, `PI_TELEMETRY`, `PI_SHARE_VIEWER_URL`.
- **Behaviour delta:** files moved to the target **with nothing left behind at `.pi`** — new
  `auth.json` and `models-store.json` went only to the target and no new default appeared.
  pi wrote plenty before it needed the network (as the probe script's comment predicts).
- **Q1/Q8:** n/a (no flag/file mechanism). **Q3/Q4/Q5/Q7:** nothing observed — see the
  question table.
- **Capability state set (recommended):**
  - ConfigIsolation: **Supported.**
  - StateIsolation: **Supported** — basis quoting
    `+ d /home/agent/probe-target/sessions`, `+ f .../models-store.json` (§6 partial-measure
    cells yes: session/`models-store` files appeared under the mechanism).
  - CredentialIsolation: **NotSupported** — basis quoting
    `+ f /home/agent/probe-target/auth.json` (this launch) and baseline
    `f /home/agent/.pi/agent/auth.json` (default): pi stores credentials in `auth.json`
    inside its own config directory, and that directory is exactly what
    `PI_CODING_AGENT_DIR` moves — so credentials travel with the config directory and no
    separate credential store exists to point anywhere else. This is §6's wider exception
    ("a probe establishes where credentials are stored"), so the basis is `measured:`,
    not `unmeasured:`. Env keys (`ANTHROPIC_API_KEY`, …) bypass the file entirely (help).
  - One `Unknown` at most ⇒ `Proven` permitted under D6/§4.1.

## kiro — Kiro CLI

- **Outcome: 5 — mechanism present but ineffective for some subtree; settings moved.** Decided
  by: `delta-env-KIRO_HOME: + f /home/agent/probe-target/settings/cli.json` (the mechanism
  worked for settings) **and**
  `+ d /home/agent/.local/share/kiro-cli/run-receipts`,
  `+ f /home/agent/.local/share/kiro-cli/data.sqlite3` — an undocumented data tree the
  variable does not move. §8.5 predicted this partial-isolation shape; the probe reproduced a
  variant of it (settings obeyed `KIRO_HOME`, data did not), which lands in §9 outcome 5 under
  its "where files went instead" branch: the data tree went instead to
  `.local/share/kiro-cli`, the location the probe script's comment documents as measured.
- **version-extracted:** `2.32.0` — transcript [kiro-2.32.0.md]. The executable is confirmed
  `kiro-cli` (§8's table row's executable).
- **Mechanism evidence:** strings: `documented KIRO_API_KEY: present`,
  `documented KIRO_HOME: present`. Neither is named in the help capture (kiro-cli's `help:`
  is a summary box listing only `chat`/`agent`/`doctor`/`settings`/`quit`); the tokens appear
  in the strings scan that `sandbox/probes/kiro.sh` runs
  (`probe_strings kiro-cli KIRO_API_KEY KIRO_HOME AWS_ACCESS_KEY_ID AWS_PROFILE`). No
  credential file appeared anywhere in the delta. The install is the vendor script
  `sh -c curl -fsSL 'https://cli.kiro.dev/install' | bash` (`exit-codes: install 0`).
- **Behaviour delta** (baseline: both `d /home/agent/.kiro` and
  `d /home/agent/.local/share/kiro-cli` **(absent)**, so every delta line is the launch's):
  at the target, `+ d /home/agent/probe-target/settings` and
  `+ f /home/agent/probe-target/settings/cli.json`; at default, the `.local/share/kiro-cli`
  tree. `~/.kiro` was never created at all — the settings moved to the target, and the only
  default-location writes are the data tree.
- **Q3: answered — partial.** `KIRO_HOME` held for the subsystem that wrote
  `settings/cli.json`, but a second subsystem ignored it
  (`.local/share/kiro-cli/data.sqlite3`, `run-receipts/`, telemetry lock). So at 2.32.0 the
  variable is **not reliably held for every subsystem**. §8.5's predicted failure shape
  ("subsystems ignore it and use `~/.kiro` regardless") did **not** match what was measured:
  no `~/.kiro` was created.
- **Capability state set (recommended):**
  - ConfigIsolation: **Supported** — basis quoting
    `+ f /home/agent/probe-target/settings/cli.json` and the default `(absent)`.
  - StateIsolation: **NotGuaranteed** — basis quoting the `.local/share/kiro-cli` subtree
    (`data.sqlite3`, `run-receipts/.reaper.lock`, `telemetry-export-drops.lock`): the state
    subsystem ignores `KIRO_HOME`.
  - CredentialIsolation: **Unknown** — `unmeasured: naming the vendor session required`.
  - One `Unknown` ⇒ `Proven` permitted under D6/§4.1; the NotGuaranteed state in the
    matrix still tells the user the mechanism leaks (§8.5's argument, kept).

## cursor — Cursor Agent CLI

- **Outcome: 1 — mechanism confirmed.** Decided by:
  `delta-env-CURSOR_CONFIG_DIR: + f /home/agent/probe-target/cli-config.json`,
  with the defaults
  `f /home/agent/.cursor/cli-config.json` (only) and `/home/agent/.config/cursor`
  **(absent)**. The mechanism moved the agent's one pre-network write into the target
  config dir; the XDG fallback location `.config/cursor` was **not** chosen.
- **version-extracted:** `2026.10.16-e3cb207` — transcript [cursor-2026.10.16-e3cb207.md];
  executable `cursor-agent` (§8 table row).
- **Mechanism evidence:** strings: `documented CURSOR_CONFIG_DIR: present`.
- **Behaviour delta:** files moved to the target — no new default locations were written.
- **Q4: unanswered.** Credential storage location was not observed: no `login`/`auth` step ran
  and no credential-bearing file name surfaced anywhere in the transcript.
- **Capability state set (recommended):**
  - ConfigIsolation: **Supported** — basis quoting `+ f /home/agent/probe-target/cli-config.json`
    and `f /home/agent/.config/cursor  (absent)`.
  - StateIsolation: **Unknown** — the agent wrote only `cli-config.json` before it needed
    network access, so no state was observable; basis quoting the absent delta tail.
  - CredentialIsolation: **Unknown** — `unmeasured:`; credential storage never located
    (§6's exception unmet).
  - Two `Unknown`s ⇒ **Experimental** under Gate C/D6.

## continue — Continue CLI

- **Outcome: 5 — mechanism present but ineffective (delta empty).** Decided by:
  `delta-flagfile---config: (no change: nothing was created, moved or removed at the watched
  locations)` while the token is in the artefacts: help lists
  `--config <path>  Configuration file for the assistant (can be a file path or hub slug)`.
- **version-extracted:** `1.5.47` — transcript [continue-1.5.47.md]; executable `cn`.
- **Mechanism evidence:** `probe_behaviour cn "$PROBE_AGENT_HOME/.continue" flagfile:--config "{}"`
  exit 0 with `{}` accepted (below) — the flag is real. strings:
  `documented CONTINUE_API_KEY: present` (per `sandbox/probes/continue.sh`'s string list),
  `documented ANTHROPIC_API_KEY: present`, `documented OPENAI_API_KEY: present`.
- **Behaviour delta:** the baseline (§7.3 step 5) already held
  `d /home/agent/.continue`, `d /home/agent/.continue/logs` and `f /home/agent/.continue/logs/cn.log`
  — created before the wrapped launch (the version/help steps, or the install) — and the delta at
  both locations was **(no change: nothing was created, moved or removed at the watched
  locations)**. So the launch against `--config <probe-target>/config.yaml` wrote nothing and
  moved nothing: the agent writes nothing otherwise before it needs the network, which the
  `sandbox/probes/continue.sh` "NO LAUNCH OVERRIDE" comment says is a measured result rather
  than an omission.
- **Candidate sweep results** (`probe_candidates cn flagfile:--config @none "" "# nothing" "{}"`
  in `sandbox/probes/continue.sh`; exits from the transcript's `exit-codes:` and contents from
  its `candidates:` section — `1: @none 2: (blank) 3: # nothing 4: {}`):
  | candidate | content tried | exit |
  |---|---|---|
  | candidate-1 | `@none` — the launch-command slot rather than a file content (control row) | 0 |
  | candidate-2 | empty file (`""`) | 0 |
  | candidate-3 | `# nothing` | 0 |
  | candidate-4 | `{}` | 0 |
- **Q8 / `INITIAL_CONFIG`:** answered — **the empty file (`""`)** is the smallest content the
  agent accepts that sets no option (exit 0); `# nothing` and `{}` were also accepted (exit 0),
  so any of the three can serve `INITIAL_CONFIG`; the smallest per §8.4 is the zero-byte
  `config.yaml`.
- **Capability state set (recommended):**
  - ConfigIsolation: **Unknown** — basis quoting the delta `(no change)` line (§9 outcome 5,
    Unknown branch: delta simply empty).
  - StateIsolation: **Unknown** — same.
  - CredentialIsolation: **Unknown** — `unmeasured:`.
  - ⇒ **Experimental** under Gate C/D6.

## amp — Amp

- **Outcome: 5 — mechanism present but ineffective.** Decided by:
  `delta-flagfile---settings-file: + d /home/agent/.local/share/amp`,
  `+ f /home/agent/.local/share/amp/accounts.json`,
  `+ f /home/agent/.local/share/amp/device-id.json`,
  `- (absent)` — the target's `settings.json` is the probe's own file (`PROBE_CONFIG_NAME=settings.json`,
  §7.3's "the probe writes a candidate file"); it was accepted
  (`behaviour-flagfile---settings-file 0`, launch `logout` per `sandbox/probes/amp.sh`) but the
  agent's *own* writes (device identity and account store) went to `.local/share/amp` and did
  **not** follow `--settings-file`. §9's outcome-5 "where files went instead" is met.
- **version-extracted:** `0.0.1792224035-gabd4f6` — transcript [amp-0.0.1792224035-gabd4f6.md].
- **Mechanism evidence:** strings: `documented AMP_API_KEY: present`,
  `documented AMP_URL: present`, `documented AMP_SETTINGS_FILE: present` — an **environment**
  equivalent (`AMP_SETTINGS_FILE`) exists that §8's table does not list (see corrections).
  NOTE: `--settings-file` does not appear verbatim in the recorded help capture
  (see the concern note below); the acceptance evidence is the exit-0 use of the flag.
- **Behaviour delta:** the default `.config/amp` was **(absent)** at the baseline and stayed
  so; the flag moved **nothing** to the target. The agent wrote only `.local/share/amp`
  (`device-id.json`, `accounts.json`).
- **Candidate sweep results** (`probe_candidates amp flagfile:--settings-file @none "" "{}"`):
  | candidate | content (`candidates:` lines) | exit |
  |---|---|---|
  | candidate-1 | the launch command's `@none` control | 0 |
  | candidate-2 | empty file (`""`) | 0 |
  | candidate-3 | `{}` | 0 |
- **Q8 / `INITIAL_CONFIG`:** answered — smallest accepted content is the **empty file (`""`)**
  (exit 0); `{}` also accepted. `INITIAL_CONFIG = ""` (a zero-byte `settings.json`).
- **Q5: answered.** `--settings-file` does **not** reach the credential/session storage:
  `accounts.json` and `device-id.json` were written at `.local/share/amp` while pointed
  at the flag's target — quoted above. (§8.4 note: Amp's mechanism stays a per-launch settings
  **file**; the storage locations are elsewhere and do not move with it.)
- **Capability state set (recommended):**
  - ConfigIsolation: **NotGuaranteed** — basis quoting
    `+ f /home/agent/.local/share/amp/device-id.json` remaining at the default and the
    `.local/share/amp` directory being the only observed write target.
  - StateIsolation: **NotGuaranteed** — basis quoting the same `device-id.json`/`accounts.json`
    lines at `.local/share/amp`.
  - CredentialIsolation: **Unknown** — `unmeasured:` (no credential file observed; `AMP_API_KEY`
    is the documented bypass).
  - One `Unknown` ⇒ `Proven` permitted under D6/§4.1 — the NotGuaranteed matrix rows are what
    tell the user the agent writes elsewhere regardless of the flag.

---

## Q1–Q8 answers

- **Q1 (Cline `CLINE_DATA_DIR` vs sandbox mode):** **unanswered.** All three cline deltas
  were `(no change ...)`. The probe cannot decide `CLINE_DATA_DIR` vs the two flags
  from this run.
- **Q2 (OpenCode config vs data):** **answered.** No OpenCode-specific data-directory variable
  exists in the strings scan (no `OPENCODE_DATA*`, no `OPENCODE_HOME`) — only
  `OPENCODE_CONFIG_DIR`, `OPENCODE_CONFIG` and the generic `XDG_DATA_HOME`.
  `OPENCODE_CONFIG` selects a config **file** that the agent **accepts a `{}`-only file under**
  (exit 0, `behaviour-envfile-OPENCODE_CONFIG 0`) and that relocates none of the agent's
  writes; `OPENCODE_CONFIG_DIR` is the one that moves config writes (transcript deltas above).
- **Q3 (Kiro `KIRO_HOME` reliability):** **answered with a finding.** The mechanism works for
  settings (`+ f .../probe-target/settings/cli.json`) but one subsystem ignores it
  (`+ f /home/agent/.local/share/kiro-cli/data.sqlite3` and neighbours). So at 2.32.0 the
  variable does not hold for every subsystem — but **not** via the shape §8.5 predicted
  (`~/.kiro` re-creation); rather the `.local/share/kiro-cli` tree. §8.5's
  basis needs correcting (see the correction duty table below).
- **Q4 (Cursor credentials under `CURSOR_CONFIG_DIR`):** **unanswered** — no credential store
  observed anywhere in the delta (no authenticated step).
- **Q5 (Amp `--settings-file` reach):** **answered.** It affects the
  settings file only; it does **not** move credentials (accounts) or the device identity:
  `+ f /home/agent/.local/share/amp/accounts.json`, `+ f .../device-id.json` while the
  settings target's `settings.json` is the probe's own file and stayed the only file there.
  (§8.4's shape note: Amp's mechanism stays `--settings-file`, a per-launch settings **file**;
  the storage is elsewhere and does not move with it.)
- **Q6 (accepted abbreviations):** **unanswered for all nine.** No abbreviated-flag candidates
  were invoked in this run (continue's and amp's candidate sweeps probed **file contents**, not
  flag spellings) — per §13/§8.4's discipline, this stays tracked debt (exact flags, D11).
- **Q7 (native named profiles):** **unanswered, with one citable candidate.** Nothing in the
  gemini/copilot/opencode/cline/pi/kiro/continue/amp help captures surfaces a named-profile
  concept. Cursor's help prints
  `--workspace <path-or-name>   Workspace directory or saved workspace name to use` — a saved,
  **named** workspace is a native named-selection concept and the only candidate for D10's
  `ProfilePresence::Known`/first named-profile constructor; whether it implies a full profile
  directory is **not** settled by this probe. Recorded as a candidate, not an answer.

## §8 correction duty (§5.1 item 4), row by row

- **gemini row:** no contradiction — `GEMINI_CLI_HOME` wrapped,
  measured nesting per §8.1. (The strings lines also contain a further
  `GEMINI_CLI_*` family; no correction to the table needed.)
- **copilot row:** **correction.** §8 lists `COPILOT_GITHUB_TOKEN` as documented bypass. The
  strings step says `documented COPILOT_GITHUB_TOKEN: ABSENT`. Only
  `GH_TOKEN` and `GITHUB_TOKEN` were found. Also the install column "npm" is
  confirmed (`npm install --global @github/copilot`, `exit-codes: install 0`).
- **opencode row:** no contradiction in the mechanism column; the install column should cite
  the package **`@opencode/cli`** (the 2.x line actually measured; `opencode-ai` 1.x is a
  parallel line, not what the probe measured — also the strings show lines
  `documented OPENCODE_CONFIG: present` verifying the second half of the row's mechanism).
- **cline row:** no contradiction (both `--config` and `--data-dir` appear in the help as
  `Configuration directory` and `Use isolated local state at this directory path`), but §8.3's
  decision questions remain unanswered (Q1) and so the shape selection stays open.
- **pi row:** no contradiction; `version-extracted` 1.3.0 from npm (the install column says
  "npm or install script", and the probe ran npm with --ignore-scripts).
- **kiro row:** **correction.** The mechanism `KIRO_HOME` held for settings writes but not the
  `.local/share/kiro-cli` tree; §8.5's predicted failure mode ("subsystems ignoring it and
  using `~/.kiro` regardless") did not match what was measured. The adapter's
  `StateIsolation` basis quotes the measured tree instead. Also the install column says
  "install script or `.deb`" for the IDE — `sh -c curl ... | bash` measured 0 and the `.deb`
  is confirmed unused (per the probe script's head comment).
- **cursor row:** no contradiction — `CURSOR_CONFIG_DIR` toggles config writes proved
  (mechanism moved `cli-config.json`), and the XDG fallback location (allowed by the design)
  never appeared.
- **continue row:** no contradiction — `--config <file>` is confirmed in help and
  accepted `{}`/empty; the executable is `cn`.
- **amp row:** **correction.** Two things the table lacks: the scan found the env equivalents
  `AMP_SETTINGS_FILE` and `AMP_HOME` present in the binary where §8's table lists only
  `--settings-file`; and the measured default-shaped writes
  (`device-id.json`, `accounts.json` under `.local/share/amp`) belong with the row, since
  `settings.json` under `.config/amp` was never written by any command this probe ran without
  credentials. `--settings-file` itself is confirmed working (exit 0 with both contents).

## Escalation — none required

No probe landed in outcome 3 and none in outcome 4, and no owner sign-off is triggered by
§5.1's item 4's "escalate outcomes 3 and 4" rule. Cline's Q1 remains an open design question
rather than an outcome-4 verdict: the deltas were ambiguous because the agent writes nothing,
not because the mechanism differs.

## Central concern for Tasks 3–11 to plan around

Of the nine transcripts, only **pi** (`PI_CODING_AGENT_DIR    - Config directory (default:
~/.pi/agent)`), **cline** (`--config <path>` and `--data-dir <path>`), and **continue**
(`--config <path>`) show the mechanism token in the `help:` capture, and **cursor** shows
only related variables (`CURSOR_API_KEY`, `CURSOR_API_ENDPOINT`). **gemini** and
**opencode** print complete help captures that do not name the env mechanism at all, **kiro**'s
is a summary box, and **copilot**'s and **amp**'s captures are truncated (`[truncated: more
than 200 lines]`) before their option lists. The mechanism tokens for gemini, copilot,
opencode, kiro, cursor and amp must therefore be cited from the `documented TOKEN: present`
strings line plus the delta, not from a help quote.
