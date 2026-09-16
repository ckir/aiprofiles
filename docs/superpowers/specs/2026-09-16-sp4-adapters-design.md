# SP4 — The remaining nine adapters: design

**Status:** draft, awaiting the adversarial panel and owner approval.
**Branch:** `sp4-adapters` (from `main` at `32e8673`).
**Oracle:** `agent-profile-implementation-spec-v3.md` (called "V3" below). Where this document and V3
disagree, V3 wins; report the conflict instead of resolving it silently.
**Previous sub-project:** SP3, `docs/superpowers/specs/2026-09-15-sp3-resolution-design.md` (merged at
`4ea5976`, released as v0.0.2).

## 1. Goal

SP4 delivers V3 §35 phase 4B: the nine remaining adapters — Gemini CLI, GitHub Copilot CLI, OpenCode,
Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI and Amp. It also closes the gap that makes
shipping them honest: today an adapter's support level and capability states are stored and tested but
never shown to a user, so an unproven adapter is indistinguishable from a proven one at the command line.

SP4 ends with twelve adapters registered, each carrying evidence that a reviewer can check against a
committed transcript, and with the report stating plainly how far each adapter is proven.

## 2. Scope

### 2.1 In scope

| Area | V3 |
|---|---|
| Rendering support level and capability states in the dry-run and `--verbose` report | §3, §26, §37 |
| A launch-path hedge for adapters that are not `Proven` | §3, §37 |
| Evidence invariants enforced by the contract suite (gates A–C, §5) | §28, §37 |
| A behavioural probe step: launch the agent under the wrapper and diff what it wrote | §2 "reverified", §28 |
| Committed evidence transcripts under `docs/evidence/` | §28, §37 |
| The Sandbox workflow accepting several agents in one dispatch | §34 |
| Nine adapters, each with metadata, contract row, end-to-end row and probe script | §2, §3, §21, §22, §28 |
| Deleting `ProfilePresence::Known` and its `status` label | §8 |

### 2.2 Out of scope

- `doctor` (SP5), including mechanism-drift detection against the recorded evidence.
- JSON output of capabilities and evidence (SP5, V3 §32).
- `create`, `list`, `delete`, `repositories` (SP5).
- Any authenticated measurement. No probe logs in to any vendor, and no secret is ever placed in CI.
- Changing the isolation mechanism of the three shipped adapters. Their metadata gains the provenance
  prefixes of D5 and nothing else.
- Native-profile adapters (see D10).

## 3. Why the visibility work belongs to SP4, not SP5

V3 parks caveat disclosure in `doctor` (§37, "doctor exposes important caveats"), and `doctor` is SP5.
Measured against the tree at `32e8673`:

- `report_lines` (`crates/agent-profile/src/output.rs:31-87`) renders `agent`, `profile`, `executable`,
  `repository`, `mechanism`, `environment`, `creates`, `arguments` and `note`. No support level and no
  capability state reaches a user.
- `doctor` is `Error::NotYetImplemented` (`crates/agent-profile/src/error.rs:191`), exit 2.
- `SupportLevel::Experimental` (`crates/agent-profile/src/adapter/metadata.rs:11`) has never shipped in a
  release build: its only user is `fake`, which is `#[cfg(debug_assertions)]`
  (`crates/agent-profile/src/adapter/mod.rs:12-13`).

So an `Experimental` adapter would today print output byte-identical to a `Proven` one. Nine adapters whose
credential isolation cannot be measured (§6) is precisely the moment that gap stops being theoretical.
V3 §37 already requires that "capabilities are separate from support level"; SP4 makes that separation
visible rather than merely typed.

## 4. Decisions and their evidence

Evidence marks: **measured** (run and observed; for this document, in the repository at `32e8673`),
**cited** (a first-party page fetched on 2026-09-16), **reasoned**.

| # | Decision | Evidence |
|---|---|---|
| D1 | SP4 runs in two phases with a measurement step between them: **SP4a** (visibility, invariants, probe harness, workflow) against code that exists now; **the probe run**, nine agents in CI, producing committed transcripts; **SP4b**, the nine adapters written against those transcripts. | reasoned: the owner's plan-vs-spec rule forbids a line-level plan whose "existing code" is an unmeasured third-party product. SP4b gets this spec now and a plan once §9's transcripts exist. |
| D2 | The report gains a `support:` line and an `isolation:` block naming each capability's state; in `ReportMode::DryRun` each capability also shows its `basis`. | measured: the report is the only surface carrying `mechanism` (`output.rs:51`); `creates:` already varies by mode (`output.rs:72-80`), so a mode-dependent expansion follows house style. |
| D3 | An adapter whose support level is not `Proven` prints one line to stderr before the agent starts, on every launch, not only in the report. | measured: the report renders only for `--dry-run` and `--verbose` (`crates/agent-profile/src/cli.rs:596`, `:605`); a plain launch prints nothing, so the report alone leaves the default path silent. |
| D4 | **Gate A.** No adapter is registered unless its mechanism token — the variable name or the flag spelling — was observed in the agent's own artefact (`--help` output, binary strings, or a first-party page) at the version recorded in `evidence.upstream_version`. Enforced by tightening `source_url` to "a URL, or the literal `measured` with non-empty `notes`". | measured: `metadata_invariants` asserts only non-emptiness today (`crates/agent-profile/tests/adapter_contract.rs:264-266`), which any placeholder satisfies. |
| D5 | **Gate B.** Every capability claim whose state is not `NotSupported` carries a `basis` that either names a specific bypass or states that a search found none. Every `basis` begins with a provenance prefix: `measured: `, `cited: ` or `unmeasured: `. | measured: this is what the shipped adapters already do informally — `claude.rs:25-27` enumerates eight override variables, `codex.rs:29-31` four, `aider.rs:41-42` names `.env` files and `AIDER_*`. The prefixes make that machine-readable for SP5's JSON output; the three shipped adapters are retrofitted. |
| D6 | **Gate C.** `SupportLevel::Proven` requires that no capability claim is `Unknown` and that `evidence.notes` is non-empty. Asserted in `metadata_invariants`, replacing the hard-coded `[claude, codex, aider]` support triple with the invariant plus an id list. | measured: the triple at `adapter_contract.rs:275-286` pins support levels by hand, so SP4 must edit it regardless; an invariant makes the rule self-enforcing for adapters ten through twelve. |
| D7 | A capability that cannot be measured because measuring it needs an authenticated session is `CapabilityState::Unknown` with an `unmeasured:` basis, and forces `Experimental` through D6. No enum variant is added. | reasoned: `NotGuaranteed` and `Conditional` are claims about a mechanism that was understood — both existing uses of `NotGuaranteed` describe a known leak path (`claude.rs:44-46`, `aider.rs:38-43`), and `Conditional` promises the conditions are enumerable (`codex.rs:43-45` lists three variables by name). Using either for "we could not log in" borrows the authority of a measurement that never happened. `NotSupported` is a positive negative claim and would defame a product that may isolate correctly. The missing information is a reason, and `basis` carries it. |
| D8 | `sandbox/probes/common.sh` gains a behavioural step: launch the agent under `agent-profile` (not `--dry-run`), then diff the container filesystem to record where the agent actually wrote. Each probe's `version`, `help`, strings excerpt and diff are committed as `docs/evidence/<agent>-<version>.md`. | measured: today `probe_agent_profile` records `cargo build`, `--version` and a dry run (`sandbox/probes/common.sh:22-26`), so it captures what `agent-profile` *plans*, never what the agent *does*; `claude.sh`, `codex.sh` and `aider.sh` stop at `--help`. Probe output lands in `target/sandbox/` (`sandbox/run.sh:68`) and `target` is gitignored (`.gitignore:4`), so nothing in the repository backs the word "measured" in an adapter comment. |
| D9 | Probes run in the Actions Sandbox workflow, which gains a list input; local Docker is the fallback, not the default. | measured: only the Podman branch of `sandbox/run.sh` creates a private ephemeral image store (`:84`) and deletes it at cleanup (`:105-108`); the Docker branch does `chmod a+rwx "$out"` and nothing else (`:88`), so agent install layers persist in the host daemon. Podman is absent from the owner's machine; the workflow sets `AGENT_PROFILE_SANDBOX_ENGINE: podman` (`.github/workflows/sandbox.yml:43`) on a runner VM that is discarded. `inputs.agent` is a single string (`:23-26`), so nine probes are nine manual dispatches. |
| D10 | `ProfilePresence::Known` and the `"known"` arm are deleted. | measured: the variant has exactly two occurrences — its declaration (`crates/agent-profile/src/adapter/metadata.rs:102`) and a `status` label (`crates/agent-profile/src/cli.rs:739`); no code constructs it. The default `presence()` can only return `Materialized` or `Absent` (`adapter/mod.rs:48-58`), so `Known` needs an adapter that identifies a profile without owning a directory. All nine mechanisms resolve to a path `agent-profile` itself creates. The repository already declined the native-profile route for Codex, the adapter V3 §2 most entitles to it (`codex.rs:69-75`, and `-p/--profile` is accepted rather than conflicting: `adapter_contract.rs:194`). cited: research on 2026-09-16 found no named-profile concept among the nine. Reversal costs one variant and one match arm. |
| D11 | An SP4 adapter ships with `conflicts: &[]` unless a probe transcript shows the option in that agent's `--help` at the pinned version. | reasoned: V3 §21 ("If the wrapper cannot prove a conflict, it must not guess"). An empty list is already a legal contract row and both `claude` and `codex` use it (`adapter_contract.rs:194-195`). Aider's list is a measured prefix-abbreviation set (`aider.rs:56-59`), which is what a proven conflict list costs; nine of those cannot come from documentation. |
| D12 | All nine ship in one wave rather than split by measurability. | cited: research on 2026-09-16 established that every one of the nine installs and runs `--help`/`--version` with no account, and every one needs an account to do real work. The proposed split axis puts all nine in the same bucket, so it separates nothing. |

## 5. The evidence bar, as contract-suite invariants

All three gates are assertions in `metadata_invariants`
(`crates/agent-profile/tests/adapter_contract.rs:241`), so they apply to every adapter automatically and a
reviewer checks a new adapter by reading one file.

```text
Gate A   source_url is a URL, or the literal "measured" with non-empty notes
Gate B   every basis starts with "measured: ", "cited: " or "unmeasured: "
Gate C   support == Proven  =>  no capability claim is Unknown, and notes is non-empty
```

Gate B's second half — that a non-`NotSupported` basis names a bypass or states none was found — is
prose, not an assertion: no test can judge whether a sentence describes a real bypass. It is a review
rule, and D8's committed transcript is what a reviewer checks it against.

**The standing red flag.** No shipped adapter claims `CredentialIsolation: Supported`: Claude is
`Conditional` (`claude.rs:38`), Codex `Conditional` (`codex.rs:42`), Aider `NotSupported` (`aider.rs:46`).
Every one of the nine has a documented API-key or token variable read from the environment regardless of
its isolation mechanism (§8). An SP4 adapter arriving with `CredentialIsolation: Supported` means the
measurement is absent or wrong, and the panel treats it as a defect.

## 6. What can and cannot be measured

The probe of D8 runs unauthenticated, which bounds each capability:

| Capability | Measurable unauthenticated? | How |
|---|---|---|
| `ConfigIsolation` | yes | set the mechanism, launch, diff: the agent's config lands under the profile directory rather than the default location. |
| `StateIsolation` | partly | history, session and log files appear in the diff only if the agent writes them before it needs the network. |
| `CredentialIsolation` | **no** | proving that stored credentials separate requires logging in to each vendor. |

Consequently `CredentialIsolation` for all nine is `Unknown` with an `unmeasured:` basis naming the
vendor session required, unless a probe shows a credential file created without authentication. By D6
this makes the adapter `Experimental`. That is the intended outcome, not a failure: it is the difference
between nine adapters diluting the standard and nine adapters each stating their own.

An adapter may still reach `Proven` later without new code: refreshing its evidence entry and capability
states from an authenticated measurement is a metadata edit. `doctor` (SP5) is where drift against the
recorded evidence is detected.

## 7. Architecture

### 7.1 Report rendering (D2)

`report_lines` gains two labels after `mechanism`, using the existing `line`/`continuation` helpers and
`LABEL_WIDTH` (`output.rs:37-38`):

```text
support:      experimental
isolation:    config: supported
              credentials: unknown
              state: not guaranteed
```

In `ReportMode::DryRun` each capability line is followed by its `basis` on a continuation line, so a user
deciding whether to trust a profile sees the reason; in `ReportMode::Verbose`, which prints on every
launch the user asked to be verbose about, only the states show.

Capability order is `Capability::ALL` (`metadata.rs:39-41`), which is fixed, so the rendering is
deterministic without sorting. State spellings are lower-case with spaces: `supported`, `not supported`,
`not guaranteed`, `conditional`, `unknown`.

### 7.2 The launch hedge (D3)

When `metadata().support` is not `Proven`, the CLI writes one line to stderr before the launcher runs:

```text
agent-profile: gemini is experimental: credential isolation is unknown. Run with --verbose for detail.
```

It names the adapter, the support level, and the first capability that is `Unknown` or `NotSupported`; if
there is none it names the support level alone. It goes to stderr so it never contaminates an agent's
stdout, and it is printed before `exec` on Unix, where nothing after the launcher runs.

### 7.3 Probe harness (D8)

`common.sh` gains `probe_behaviour <agent>`: run the agent under `agent-profile` with a throwaway
application root and a non-interactive argument (`--version` where the agent accepts it after the
wrapper's own arguments), then record the set of paths created under both the profile directory and the
agent's default location. The existing `diff.txt` (`sandbox/run.sh:154`) captures container filesystem
changes; the new step narrows that to the two directories that answer the question.

A probe that cannot run the agent non-interactively records that fact and stops; it never waits for
input, and it never supplies a credential.

### 7.4 Evidence transcripts (D8)

`docs/evidence/<agent>-<version>.md` holds, for one probe run: the install command, `--version` output,
the `--help` excerpt containing the mechanism token, the strings excerpt listing candidate bypass
variables, and the behavioural diff. The adapter's `evidence.upstream_version` must match the file's
version, which is what makes Gate A checkable by a reviewer rather than by trust.

### 7.5 Workflow (D9)

`sandbox.yml`'s `agent` input accepts a whitespace- or comma-separated list and loops, so one dispatch
probes all nine. The `pull_request` trigger keeps running `test` mode only, and no secret is added.

## 8. The nine adapters

Every row below is **cited** — established on 2026-09-16 from first-party documentation or repository
source — and none is yet **measured**. The probe run of §9 converts these into evidence; where a probe
contradicts a row, the probe wins and this table is corrected rather than the adapter bent to fit it.

| Agent | id | Executable | Mechanism (claimed) | Shape | Documented credential bypass |
|---|---|---|---|---|---|
| Gemini CLI | `gemini` | `gemini` | `GEMINI_CLI_HOME` | env dir | `GEMINI_API_KEY`, `GOOGLE_API_KEY`, `GOOGLE_APPLICATION_CREDENTIALS` |
| GitHub Copilot CLI | `copilot` | `copilot` | `COPILOT_HOME` | env dir | `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_TOKEN` |
| OpenCode | `opencode` | `opencode` | `OPENCODE_CONFIG_DIR` | env dir | provider API keys; credentials live outside the config directory (§8.2) |
| Cline CLI | `cline` | `cline` | `--config` and `--data-dir`, or `CLINE_DATA_DIR` | undecided (§8.3) | `ANTHROPIC_API_KEY` |
| Pi | `pi` | `pi` | `PI_CODING_AGENT_DIR` | env dir | provider keys, unless `auth.json` is present |
| Kiro CLI | `kiro` | `kiro-cli` | `KIRO_HOME` | env dir | `KIRO_API_KEY` |
| Cursor Agent CLI | `cursor` | `cursor-agent` | `CURSOR_CONFIG_DIR` | env dir | `CURSOR_API_KEY` |
| Continue CLI | `continue` | `cn` | `--config <file>` | config file arg | `CONTINUE_API_KEY`; secrets live in a separate `.env` |
| Amp | `amp` | `amp` | `--settings-file <file>` | config file arg | `AMP_API_KEY`; workspace and managed settings override |

Agent ids are what a user types (`agent-profile gemini work`) and must satisfy `AgentId::parse`
(`adapter_contract.rs:247-250`). Executable names are the binary the adapter discovers, and differ from
the id for Kiro, Cursor and Continue.

### 8.1 Gemini nests

`GEMINI_CLI_HOME` names a directory *containing* `.gemini`, not the configuration directory itself: with
`GEMINI_CLI_HOME=<dir>`, Gemini reads `<dir>/.gemini/`. A file written directly at `<dir>` is ignored
silently. The adapter therefore declares the profile directory and lets the agent create `.gemini` inside
it; `paths()` must not declare `<dir>/.gemini`, because `agent-profile` creates only what it owns and an
empty `.gemini` would misrepresent a profile as materialized. The probe confirms the nesting.

### 8.2 OpenCode splits config from credentials

`OPENCODE_CONFIG_DIR` governs configuration only; `auth.json` and session data live under
`XDG_DATA_HOME` (default `~/.local/share/opencode`). Setting `XDG_DATA_HOME` would isolate them, but that
variable is not OpenCode's — it redirects every XDG-aware program in the launched process tree, which
exceeds what an adapter may claim to control. Unless the probe establishes an OpenCode-specific data
variable, the adapter sets `OPENCODE_CONFIG_DIR` alone and declares `CredentialIsolation: NotSupported`
and `StateIsolation: NotSupported`, each with a basis naming the directory that does not move. This is the
one place where V3's table (§2, "Config/home isolation") is more confident than the evidence, and the
conflict is reported here rather than resolved silently.

### 8.3 Cline's mechanism is undecided by design

`--config` selects a settings directory and `--data-dir` an isolated state directory, but `--data-dir` is
documented as also enabling sandbox mode — an isolation flag that changes agent behaviour beyond
isolation. `CLINE_DATA_DIR` is a documented environment equivalent that may not carry that side effect.
The probe answers three questions in order: does `CLINE_DATA_DIR` isolate state; does it enable sandbox
mode; does `--config` work alongside it. The answer selects one of three shapes:

1. `CLINE_DATA_DIR` alone — the existing `env_dir_plan`, no new helper.
2. `CLINE_DATA_DIR` plus `--config` — a new helper combining one variable and one flag.
3. `--config` plus `--data-dir` — a new helper emitting two flag/value pairs.

`config_file_arg_plan` cannot express 2 or 3: it emits exactly one flag/value pair
(`adapter/mod.rs:197`) drawn from the first `File`-kind path (`:191-195`). Whether SP4b adds a helper, and
which, is settled by measurement. No helper is written speculatively.

### 8.4 The two configuration-file adapters need a minimal accepted file

Continue and Amp are selected by a file argument, so `agent-profile` must create that file before the
agent reads it, and the file's initial contents are part of the mechanism. Aider set the precedent and the
cost of guessing: a missing, empty or comment-only `.aider.conf.yml` each exits 2, and `{}` was accepted
only because it was measured (`aider.rs:20-21`, SP2 design D5). The same question is open for Continue's
`config.yaml` and Amp's `settings.json`, and the answer must be the smallest content the agent accepts
that sets no option — `agent-profile` never invents configuration (V3 §9). Until a probe establishes it,
no contents are written into the spec.

### 8.5 Kiro's mechanism is documented as unreliable

`KIRO_HOME` is documented, but an open upstream report describes subsystems ignoring it and using
`~/.kiro` regardless, failing silently. If the probe reproduces that, the adapter ships `Experimental`
with `ConfigIsolation: NotGuaranteed` and a basis citing the specific subsystems, which is exactly the
state `NotGuaranteed` exists for. Kiro CLI is the rebrand of Amazon Q Developer CLI; the executable is
`kiro-cli`.

## 9. The probe run

Between SP4a and SP4b, one dispatch of the Sandbox workflow probes all nine. Its output is nine
`docs/evidence/` files, committed in one commit, and a correction pass over §8 for every row a probe
contradicted.

A probe that fails — the agent will not install, or will not run non-interactively — is recorded as such,
and its adapter is written from cited evidence with `ConfigIsolation: Unknown`, which forces
`Experimental`. No adapter is dropped for a failed probe, and no probe failure is worked around by
installing the agent on the owner's machine.

## 10. Testing (V3 §34)

| Suite | Addition |
|---|---|
| `tests/adapter_contract.rs` | gates A–C in `metadata_invariants`; one `expected()` row per new adapter (the `panic!` at `:131` makes a missing row a failure, not a silent pass); one `conflict_contract` row each. |
| `tests/adapters_e2e.rs` | one row per new adapter, launching the renamed `fake-agent`, proving the plan reaches the child. No real agent runs in CI. |
| `crates/agent-profile/src/output.rs` unit tests | the `support:` and `isolation:` rendering for each `CapabilityState` spelling, and that `basis` shows in `DryRun` and not in `Verbose`. |
| `tests/launch.rs` | the D3 stderr hedge: present for an `Experimental` adapter, absent for a `Proven` one, on stderr and not stdout. |
| Deletion of `Known` | the `status` presence labels stay covered by the existing SP3 resolution tests; no test may assert the string `known` afterwards. |

Every new test is proven non-vacuous with a logic mutant, per the repository's assertion-strength
discipline: a test that cannot fail is a test that certifies nothing.

## 11. Documentation

- `README.md`: the supported-agent table grows to twelve, with a support-level column.
- `CONTRIBUTING.md`: how to add an adapter, pointing at the gates and at `docs/evidence/`.
- `docs/evidence/README.md`: what a transcript is for and how to refresh one.
- `ROADMAP.md`: SP4 done, SP5 next; the `ProfilePresence::Known` note added at `8741ea0` is resolved.
- `TODO.md`: any capability left `Unknown` that an authenticated measurement could settle is tracked debt.

## 12. Known limits

- **Credential isolation is unproven for all nine.** SP4 states this rather than fixing it; fixing it
  needs a paid account per vendor.
- **Evidence is a snapshot.** A transcript pins one version on one day. Drift detection is `doctor` (SP5),
  and until then a mechanism can change upstream without the repository noticing.
- **The behavioural probe proves where files landed, not that nothing leaked.** A diff shows the config
  directory moved; it cannot show that no credential was read from a shared location.
- **`StateIsolation` measurement is partial** for agents that write history only after a network call.
- **Nine adapters share one probe run.** If the workflow is changed between SP4a and the probe run, the
  transcripts and the harness can disagree; the transcript records the harness commit.
- **Most of the nine are npm-installed, and on Windows that means a shim.** SP2 refuses a shim with a
  detection message and a fix hint rather than parsing it (SP2 design D2, V3 §23.2), so on Windows these
  adapters need an `executable` entry in the configuration pointing at the real binary. This is existing
  behaviour, not new to SP4, but it now affects most adapters instead of one, and the README must say so.
- **The probe measures Linux only.** The Sandbox workflow runs a Linux container, so mechanism evidence is
  Linux evidence. Where an agent resolves its home differently on Windows or macOS, the adapter's claim is
  weaker than the transcript suggests; `doctor` (SP5) is where a per-platform check belongs.

## 13. Open questions, and where each is resolved

| # | Question | Resolved by |
|---|---|---|
| Q1 | Does `CLINE_DATA_DIR` isolate state without enabling sandbox mode? | probe run (§9); selects the shape in §8.3 |
| Q2 | Does an OpenCode-specific data directory variable exist? | probe run; decides §8.2's capability states |
| Q3 | Does `KIRO_HOME` hold for all subsystems at the probed version? | probe run; decides Kiro's `ConfigIsolation` |
| Q4 | Do Cursor's stored credentials live under `CURSOR_CONFIG_DIR`? | probe run if observable unauthenticated; otherwise `Unknown` |
| Q5 | Does `--settings-file` affect Amp's credential or session storage? | probe run; otherwise `Unknown` |
| Q6 | Which agents expose a conflicting option in `--help` at the pinned version? | probe transcripts; D11 defaults to `&[]` |
| Q7 | Does any of the nine have a native named-profile concept? | probe run; if one does, D10's deletion is reversed in SP4b |
| Q8 | What is the smallest file content Continue and Amp each accept that sets no option? | probe run (§8.4); no contents are guessed |

No question in this table is answered by reasoning in SP4b's plan. Each is answered by a transcript or
recorded as unanswered, and an unanswered question produces `Unknown`, never a guess.
