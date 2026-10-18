# SP4b The Nine Adapters Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Register the nine remaining adapters — `gemini`, `copilot`, `opencode`, `cline`, `pi`, `kiro`,
`cursor`, `continue`, `amp` — each resting on a committed, CI-verified transcript of its own probe run, so
SP4 ends with twelve adapters and V3 §35 phase 4B is complete.

**Architecture:** No adapter is written before its measurement exists. The probe run (design §9) produces one
transcript per agent; each adapter then lands as one commit holding the transcript and its registration
together — Gate A requires a transcript for every registered adapter, and the evidence allow-list rejects a
transcript whose id is not yet registered, so the two halves can never be separate commits. Capability states
are read off the transcript, never reasoned into it; `src/adapter/gate.rs` checks the shape and §5.1's manual
surface is what a reviewer checks the content against.

**Tech Stack:** Rust 1.98 (edition 2024), POSIX shell, GitHub Actions. No new dependencies.

**Design (the oracle):** `docs/superpowers/specs/2026-09-16-sp4-adapters-design.md`; V3
(`agent-profile-implementation-spec-v3.md`) wins over it and §1.1 of the design records every divergence.
Execution plan conventions follow `docs/superpowers/plans/2026-09-16-sp4a-adapter-evidence.md`.

**Not in this plan:** `doctor`, JSON output, `create`/`list`/`delete`/`repositories` (all SP5); any
authenticated measurement (§2.2) — no adapter ships `CredentialIsolation: Supported`; changes to the three
shipped adapters' mechanisms.

---

## How this plan differs from SP4a's, and why

SP4a's plan was byte-exact: every file was read out of a working prototype. **This one cannot be.** Design
§13: "No question in this table is answered by reasoning in SP4b's plan. Each is answered by a transcript,
an owner ruling, or recorded as unanswered." The versions, the capability states and the `basis` sentences
come from measurements that do not exist until Task 2 runs. So this plan pins everything that is already
decided — ids, executables, mechanism variables and flags, file names, registration points, contract rows,
test names — and gives each measurement-dependent field a decision rule keyed to the probe outcome. The
gates are the verifier; a task is done when the suite is green **and** the decision rules were followed, not
when a code block was copied.

Rules for every task:

1. **Step 0 — state check.** `git status --short` prints nothing, and `HEAD` is the previous task's commit.
   If any check fails, STOP and report `STATE_MISMATCH: <what>`.
2. **Oracle.** The gates pin the shape; the design is the oracle behind them. If a check fails, fix the code
   to match the design; never edit a check to match the code.
3. **Gate.** Run the gate exactly as written; do not add flags.
4. **Claim the weakest state the evidence supports** (CONTRIBUTING "Adding an adapter"). `NotSupported` with
   a reason beats `Supported` with a hope. An unanswered question produces `Unknown`, never a guess.
5. **A transcript is never edited.** It is committed byte for byte from the CI artifact, or the probe script
   is fixed and the probe re-runs. The trailing newline and the two-space body indent are load-bearing
   (§7.4, and the `field()` coupling in ROADMAP "Tracked debt").
6. **Same commit, always.** The transcript and the adapter's registration land in one commit. Separate
   commits leave an intermediate state where either Gate A or the evidence allow-list is red.
7. **No real agents on the host.** Probes run only in the disposable container or in the Actions Sandbox
   workflow. No secret reaches any probe job (`permissions: contents: read` is a security boundary).
8. **Toolchain drift.** `rust-toolchain.toml` pins `stable`. If the gate fails with a diagnostic in code
   written from this plan, STOP and report it; do not silence it.
9. **Mutant.** Where a task names a mutant, apply it, confirm the named check goes red, restore with
   `git checkout -- <file>`, and confirm a clean tree. A test that cannot fail certifies nothing.

Gate commands (every task, after its own checks):

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
typos
find sandbox -name '*.sh' -exec shellcheck -s sh {} +
cargo nextest run --workspace --no-tests=pass
just probe-tests
```

Expected: `fmt`, `typos` and `shellcheck` print nothing; clippy prints no warnings; nextest ends with all
passed; every shell suite ends with `all ... checks passed`.

## File structure

| File | Task | Responsibility |
|---|---|---|
| `sandbox/probes/common.sh` | 1 | `probe_candidates` accumulates instead of destroying the first sweep |
| `sandbox/tests/probe-harness.sh` | 1 | the accumulating-sweep tests |
| (CI, human) Actions → Sandbox | 2 | the probe run for the nine, and the triage ledger |
| `crates/agent-profile/src/adapter/<id>.rs` | 3–11 | one module per new adapter |
| `crates/agent-profile/src/adapter/mod.rs` | 3–11 | `mod`, `pub use`, `REAL_ADAPTERS` per adapter |
| `crates/agent-profile/tests/adapter_contract.rs` | 3–11 | `expected()` row, `conflict_contract` row, support table row |
| `crates/agent-profile/tests/adapters_e2e.rs` | 3–11 | env list row, or a bespoke test for flag adapters |
| `crates/agent-profile/tests/launch.rs` | 3–11 | the known-agents string at `:245`, extended per adapter |
| `docs/evidence/<id>-<version>.md` | 3–11 | the transcript, committed byte for byte |
| `README.md` | 3–11, 12 | rows move from "Planned adapters" to the supported table |
| `ROADMAP.md`, `TODO.md`, `CONTRIBUTING.md` | 12 | SP4 closeout |

## The adapter task template

Tasks 3–11 share one shape. This template is the shared part; each task below adds only what is its own.
Every step ends in a green suite.

- [ ] **Step 0: State check** (rule 1).
- [ ] **Step 1: Commit the transcript.** Copy `sandbox-transcript-<id>/<id>-<version>.md` from the CI
      artifact to `docs/evidence/<id>-<version>.md`, unmodified, where `<version>` is the artifact's
      `version-extracted` value (the first `[A-Za-z0-9._-]+` run in `--version` output that contains a
      digit — §7.3 step 2). Do not rename, reformat, or fix the trailing newline.
- [ ] **Step 2: Write `crates/agent-profile/src/adapter/<id>.rs`** — module doc comment naming the
      mechanism and the design §8 subsection; the variable/flag `const`; `static METADATA`; the unit struct;
      `impl Adapter` with `metadata()`, `paths()` and `plan()`, following `aider.rs` (flag/file mechanisms)
      or `claude.rs` (env mechanisms) as the pattern. Field rules:
      - `id`, `executable`, and the `Mechanism` payload: pinned in the task below. They are not negotiable
        against the probe script — `sandbox/probes/<id>.sh` already exists and its mechanism token must
        equal `Mechanism::probe_token()`.
      - `upstream_version`: the transcript's `version-extracted`, copied exactly. `verified_at`: the probe
        run's date, ISO `YYYY-MM-DD`.
      - `source_url`: the first-party documentation URL, or the literal `"measured"` when the transcript is
        the sole evidence (aider precedent). Gate A accepts either.
      - `evidence.notes`: what the transcript establishes, in one line. Non-empty always; Gate C requires it
        for `Proven`.
      - `capabilities`: exactly one claim per `Capability::ALL` entry, each `basis` starting `measured: `,
        `cited: ` or `unmeasured: ` — with `unmeasured: ` **iff** the state is `Unknown` (Gate B's
        biconditional). States come from the task's decision rules, never from §8's table: the transcript
        wins over the table, and the table is corrected in Task 12 if the probe contradicted it.
      - `env`: the mechanism variable with `sensitive: false`, plus any variable the adapter's plan sets.
        Credential-bypass variables are **not** `EnvOverride`s (Claude precedent) — they live in
        `evidence.notes` and in the credential claim's `basis`.
      - `conflicts`: `&[]` unless the transcript measured an abbreviated spelling of the mechanism flag
        (Q6), in which case the abbreviations join the `ConflictOption`'s `long` array, Aider-style. The
        mechanism's own option is auto-scanned by `check_conflicts` and must **not** be repeated here
        (`aider.rs:59-63` records why: a duplicate masks the chain).
- [ ] **Step 3: Register it.** In `crates/agent-profile/src/adapter/mod.rs`: add `mod <id>;` (lines 10–14),
      `pub use <id>::<Id>;` (lines 27–31), and append `&<Id>` to `REAL_ADAPTERS` (line 113) — registry
      order is what the CLI, the manifest example and the known-agents error string all use.
- [ ] **Step 4: Contract rows.** In `tests/adapter_contract.rs`: an `expected()` row in the `match id`
      block (env adapters use the `env_dir(var, notes)` closure; flag adapters copy the `aider` row with
      their own flag, file name and `INITIAL_CONFIG`); a `conflict_contract` row (`(&[], &[...])` is a
      valid shape); a row in the pinned support table at lines 404–415, matching the `support` field
      written in Step 2.
- [ ] **Step 5: End-to-end row.** In `tests/adapters_e2e.rs`: env adapters extend the list at line 35 to
      `(id, executable, var)` — the executable, because `bin_with` must copy `fake-agent` under the name
      the adapter launches, and three ids differ from it — and the loop body uses that third field for
      `FAKE_AGENT_ECHO_ENV`. Flag adapters get a bespoke test in the `aider` pattern.
- [ ] **Step 6: Known-agents string.** In `tests/launch.rs:245`, extend the expected list with the new id,
      in `REAL_ADAPTERS` order, before `fake`. This breaks on every adapter task and is never optional.
- [ ] **Step 7: README row.** Move the agent's row from "Planned adapters" to the supported table, with the
      support level **and** the three capability states in the same row (§6.1's binding rule: a support
      level never appears without its capabilities). Use the probed version in the agent label.
- [ ] **Step 8: Run the gate** (gate commands above). Expected: all green.
- [ ] **Step 9: Commit** — transcript, module, registration, rows, string, README in one commit:
      `feat(adapter): add <id> with measured evidence at <version>`.
- [ ] **Step 10: Mutant** (rule 9) — change `Mechanism::Env(VAR)` / `Mechanism::FlagFile(OPTION)` to name a
      wrong variable or flag, run `cargo nextest run -E 'test(the_planned_mechanism_is_the_declared_one) or
      test(gate_a_every_transcript_contains_its_adapter_mechanism_token)'`. Expected: at least one RED.
      Restore and re-check the gate.

---

### Task 1: `probe_candidates` accumulates instead of destroying the first sweep

Close the ROADMAP "Tracked debt" entry before the nine probe scripts gain any second sweep. The function
resets its counter and truncates `candidates.txt` on every call (`sandbox/probes/common.sh:1147-1148`), so a
second sweep in one script silently overwrites the first sweep's artefacts.

**Files:**
- Modify: `sandbox/probes/common.sh` — `probe_candidates` (line 1143).
- Modify: `sandbox/tests/probe-harness.sh` — the sweep tests (lines 936–1055).

**Interfaces:**
- Produces: `probe_candidates <executable> <mechanism> <candidates...>` — repeatable within one probe
  script; each call appends to `candidates.txt` and continues the step-id sequence (`candidate-N`).
  Existing single-sweep callers (`aider.sh:59`, `amp.sh:67`, `continue.sh:35`) are unaffected: one call
  behaves exactly as before.

- [ ] **Step 0: State check** (rule 1).
- [ ] **Step 1: Write the failing test** in `sandbox/tests/probe-harness.sh` — a fixture probe that calls
      `probe_candidates` twice with different candidates; assert `candidates.txt` holds both sweeps' lines
      in order, and that the second sweep's artefacts are named `candidate-3`…, not `candidate-1`… again.
- [ ] **Step 2: Run it to verify it fails.** `just probe-tests`. Expected: the new check RED, everything
      else green.
- [ ] **Step 3: Fix `probe_candidates`** — delete the counter reset and the truncation so a call appends.
      Keep the per-candidate `.cmd`/`.exit-code`/`.txt` naming driven by the now-monotonic counter.
- [ ] **Step 4: Run the suite.** `just probe-tests`. Expected: all checks passed.
- [ ] **Step 5: Run the gate.** Expected: all green.
- [ ] **Step 6: Commit.** `fix(sandbox): make probe_candidates accumulate across sweeps`.
- [ ] **Step 7: Record the capstone consequence.** Per ROADMAP, this source change invalidates the capstone
      over the `common.sh`/`probe-harness.sh` delta. Note it in the pull request description so the audit
      round covers exactly this range; do not silently proceed past an owner who wants the audit first.

### Task 2: The probe run for the nine, and the triage ledger

Dispatch the Sandbox workflow over the nine agents, read every transcript, and record what each probe
established. This is the task every later task reads from; it is mostly maintainer work, and its output is a
ledger file, not code.

**Files:**
- Create: `docs/superpowers/plans/2026-10-17-sp4b-probe-triage.md` — the ledger. Never inside
  `docs/evidence/`: the allow-list rejects any file there that is not a registered adapter's transcript.
- Create: `docs/evidence/<id>-<version>.md` × 9 — not yet committed; downloaded and held for Tasks 3–11.
  Holding them out of the tree until their adapter lands keeps the allow-list green at every commit.

- [ ] **Step 0: State check** (rule 1). `sandbox/probes/` must contain all nine scripts.
- [ ] **Step 1: Dispatch.** Actions → Sandbox → Run workflow, `mode: probe`, `agents: gemini copilot
      opencode cline pi kiro cursor continue amp`. No secret is added; the probe job keeps
      `permissions: contents: read`.
- [ ] **Step 2: Download** each job's `sandbox-transcript-<id>` artifact. One per agent; nine total.
- [ ] **Step 3: Read each transcript and classify it** under design §9's five outcomes — 1 confirmed,
      2 probe failed, 3 mechanism disproved, 4 mechanism differs, 5 present but ineffective. Record in the
      ledger, per agent: the outcome, the `version-extracted`, the observed behaviour delta, and the
      answers to the design's open questions it can answer (Q1 Cline, Q2 OpenCode, Q3 Kiro, Q4 Cursor,
      Q5 Amp, Q6 abbreviations, Q7 native profiles, Q8 minimal config file).
- [ ] **Step 4: Perform §8's correction duty** (§5.1 item 4): re-read the design's §8 table against the
      transcripts and record every contradiction in the ledger. The probe wins; the adapter is written
      against what was measured, and Task 12 corrects the table.
- [ ] **Step 5: Escalate outcomes 3 and 4.** An outcome 3 (mechanism disproved) means the adapter is **not
      registered**, its row moves to `TODO.md` as tracked debt, and — because that reduces V0.1 below V3's
      twelve adapters — **the owner signs off before SP4b proceeds**. An outcome 4 needs an owner look too:
      §8's mechanism may not match what the agent implements, and the closed `Mechanism` enum may not
      express it (see Task 11's stop rule).
- [ ] **Step 6: Commit the ledger.** `docs: record the SP4b probe triage`. Nothing under `docs/evidence/`
      is committed here.

**Checkpoint: probe triage reviewed**
- [ ] Nine transcripts downloaded; each classified under exactly one outcome.
- [ ] Q1–Q8 answered or recorded unanswered; §8's correction duty performed.
- [ ] Owner sign-off obtained for any outcome-3 or outcome-4 adapter.
- [ ] The artifact retention window (90 days) is noted in the ledger — Tasks 3–11 must land inside it, or
      the run must be repeated.

### Task 3: Gemini CLI (`gemini`)

Pinned: `id: "gemini"`, `executable: "gemini"`, `Mechanism::Env("GEMINI_CLI_HOME")`
(`sandbox/probes/gemini.sh:27`, token `env:GEMINI_CLI_HOME`). Install is npm (`@google/gemini-cli`).

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **§8.1 nesting.** `paths()` declares the profile directory only. `GEMINI_CLI_HOME` names a directory
      *containing* `.gemini`; declaring `<dir>/.gemini` would have `agent-profile` create a directory that
      belongs to the agent (V3 §9). The transcript's delta confirms or contradicts the nesting; if it does,
      record it in the ledger and stop.
- [ ] **Credential bypass** (from `gemini.sh:17` and the transcript): `GEMINI_API_KEY`, `GOOGLE_API_KEY`,
      `GOOGLE_APPLICATION_CREDENTIALS`. State is `Unknown` with an `unmeasured: ` basis naming the vendor
      session required, unless the probe located credential storage (§6's wider exception), which permits
      `NotSupported`/`Conditional` with a `measured: ` basis instead.
- [ ] **Config and state states** come from the delta: files moved → `Supported` with a `measured: ` basis;
      nothing written before the network → `StateIsolation: Unknown` (§9 outcome 5 rule).

### Task 4: GitHub Copilot CLI (`copilot`)

Pinned: `id: "copilot"`, `executable: "copilot"`, `Mechanism::Env("COPILOT_HOME")`
(`sandbox/probes/copilot.sh:41`, token `env:COPILOT_HOME`). Install is npm (`@github/copilot`).

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **Credential bypass** (`copilot.sh:14`): `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_TOKEN` — default
      `Unknown`/`unmeasured: ` per Task 3's rule.
- [ ] **States** from the delta, as in Task 3.

### Task 5: OpenCode (`opencode`)

Pinned: `id: "opencode"`, `executable: "opencode"`, `Mechanism::Env("OPENCODE_CONFIG_DIR")`
(`sandbox/probes/opencode.sh:39`, token `env:OPENCODE_CONFIG_DIR`). The probe also tests
`envfile:OPENCODE_CONFIG` (`opencode.sh:58`); that second measurement informs the claims, but the adapter's
mechanism is the directory variable — one adapter, one `Mechanism`.

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **Q2 decides the credential and state claims** (§8.2): if the probe found no OpenCode-specific data
      directory variable, `CredentialIsolation` and `StateIsolation` are `NotSupported`, each with a basis
      naming the directory that does not move (`auth.json` and session data under `XDG_DATA_HOME`, default
      `~/.local/share/opencode`). Two non-`Unknown` negatives do not block `Proven` under D6 — that is
      exactly §8.2's point, so do not soften them to `Unknown` out of caution; a `measured: ` basis is
      what permits the state.
- [ ] **If the probe found a data variable**, the states follow what it showed, and §8.2's row is corrected
      in Task 12.

### Task 6: Pi (`pi`)

Pinned: `id: "pi"`, `executable: "pi"`, `Mechanism::Env("PI_CODING_AGENT_DIR")`
(`sandbox/probes/pi.sh:47`, token `env:PI_CODING_AGENT_DIR`). Install is npm
(`@earendil-works/pi-coding-agent`, `--ignore-scripts`).

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **Credential bypass** (`pi.sh:26`): provider keys, "unless `auth.json` is present" (§8) — the
      transcript's `strings:` and delta decide the exact basis wording.

### Task 7: Kiro CLI (`kiro`)

Pinned: `id: "kiro"`, `executable: "kiro-cli"` — the id and executable differ (§8); probe scripts,
transcripts and artefacts are named by **id**, the launched binary by executable.
`Mechanism::Env("KIRO_HOME")` (`sandbox/probes/kiro.sh:35`, token `env:KIRO_HOME`). Install is a vendor
script (`https://cli.kiro.dev/install`).

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **Q3 decides `ConfigIsolation`** (§8.5): `KIRO_HOME` is documented but an open upstream report
      describes subsystems ignoring it and failing silently. If the probe reproduces that, the state is
      `NotGuaranteed` with a basis citing the specific subsystems — that is the state `NotGuaranteed`
      exists for. Combined with one `Unknown` credential claim, D6 still permits `Proven`.
- [ ] **A script-installed agent cannot pin a version** (§12): the probe refuses a version argument for
      Kiro rather than dropping it, so the transcript's `install:` line names whatever the vendor served.
      The registry's `upstream_version` is the `version-extracted` value, nothing else.

### Task 8: Cursor Agent CLI (`cursor`)

Pinned: `id: "cursor"`, `executable: "cursor-agent"`, `Mechanism::Env("CURSOR_CONFIG_DIR")`
(`sandbox/probes/cursor.sh:56`, token `env:CURSOR_CONFIG_DIR`). Install is a vendor script
(`https://cursor.com/install`).

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **Q4** (§13): if the probe cannot observe where stored credentials live unauthenticated, the claim is
      `Unknown` with an `unmeasured: ` basis — never a guess in either direction.
- [ ] **Version pinning** as in Task 7 (script install).

### Task 9: Continue CLI (`continue`)

Pinned: `id: "continue"`, `executable: "cn"`, `Mechanism::FlagFile(--config)` with
`ConflictOption { long: &["--config"], short: None }` (`sandbox/probes/continue.sh:34`, token
`flagfile:--config`). Config file name: `config.yaml` (`PROBE_CONFIG_NAME`, `continue.sh:17`). Install is
npm (`@continuedev/cli`).

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **`INITIAL_CONFIG` is measured, not guessed** (§8.4): the transcript's `candidates:` section records
      each candidate content's exit code (`continue.sh:35` sweeps `""`, `"# nothing"`, `"{}"`). Take the
      smallest content the agent accepts that sets no option, exactly as Aider's `{}` was established. If
      no candidate was accepted, stop and record it in the ledger — a config-file adapter with no
      acceptable initial file cannot ship its mechanism.
- [ ] **Q6 abbreviations**: if the probe measured which prefixes `cn` accepts, they join the `long` array
      (`"--confi"`, …). If not recorded, ship the exact flag only and the unanswered question becomes
      tracked debt in Task 12.
- [ ] **`paths()`** declares the directory and the file, Aider-style: `(dir, Dir)`, `(dir.join("config.yaml"),
      File { contents: INITIAL_CONFIG })`.
- [ ] **Bespoke e2e test** in the `aider` pattern (`adapters_e2e.rs:62-181`): assert the planned argv is
      `["--config", <profile>/config.yaml, …user args]` and the file is created with `INITIAL_CONFIG`.

### Task 10: Amp (`amp`)

Pinned: `id: "amp"`, `executable: "amp"`, `Mechanism::FlagFile(--settings-file)` with
`ConflictOption { long: &["--settings-file"], short: None }` (`sandbox/probes/amp.sh:66`, token
`flagfile:--settings-file`). Config file name: `settings.json` (`amp.sh:17`). Install is npm
(`@ampcode/cli`).

- [ ] Follow the adapter task template. Additional decisions:
- [ ] **`INITIAL_CONFIG`** from the transcript's `candidates:` (`amp.sh:67` sweeps `""`, `"{}"`), as in
      Task 9. Q8 is answered per adapter; Aider's `{}` is not transferable.
- [ ] **Q5**: whether `--settings-file` affects credential or session storage decides the credential and
      state claims; unobservable → `Unknown`, `unmeasured: `.
- [ ] **Bespoke e2e test**, as in Task 9, with Amp's flag and file name.

### Task 11: Cline CLI (`cline`) — and the stop rule

Pinned: `id: "cline"`, `executable: "cline"` (`sandbox/probes/cline.sh`). The probe tests three mechanisms
independently: `env:CLINE_DATA_DIR`, `flagdir:--data-dir`, `flagdir:--config` (`cline.sh:29-31`). Install is
npm (`cline`).

- [ ] **Step 0: Read Q1's answer in the ledger** (Task 2). It selects one of §8.3's three shapes:
      1. `CLINE_DATA_DIR` alone → `Mechanism::Env("CLINE_DATA_DIR")`. A divergence from V3 §2, already
         recorded in design §1.1; nothing new to report.
      2. `CLINE_DATA_DIR` plus `--config`, or
      3. `--config` plus `--data-dir` → **STOP.** The `Mechanism` enum is closed
         (`metadata.rs:135-144`: `Env | EnvFile | FlagDir | FlagFile`) and one adapter carries exactly one
         mechanism, so a two-part shape is not expressible without a public-enum change. Report
         `SHAPE_NOT_EXPRESSIBLE: <the measured shape>` and wait for the owner: the choices are extending
         `Mechanism` (a public API change with a `probe_token` and a `common.sh` `probe_apply` arm to
         match), or shipping shape 1's nearest honest single mechanism with the second factor documented.
         Design §8.3 forbids writing a speculative helper, and §13 forbids answering this by reasoning.
- [ ] **Steps 1–10 of the template**, with:
- [ ] **`--data-dir`'s sandbox side effect** (§8.3): if shape 3 is chosen and the flag is shipped, a
      user-supplied second spelling of `--data-dir` does not merely defeat isolation — it flips sandbox
      mode — so both flags are declared, and the mechanism's report sentence must name both. Under the
      closed enum that requirement is another reason shape 3 needs the owner's ruling first.
- [ ] **Credential bypass** (`cline.sh:21`): `ANTHROPIC_API_KEY` (and whatever the transcript's `strings:`
      section shows beyond it), default `Unknown`/`unmeasured: `.
- [ ] **e2e row**: the env list for shape 1; a bespoke test for shapes 2 and 3.

### Task 12: Documentation and SP4 closeout

**Files:** `README.md`, `ROADMAP.md`, `TODO.md`, `CONTRIBUTING.md`, the design's §8 table.

- [ ] **Step 0: State check** (rule 1); Tasks 3–11 all landed, or the ledger names the adapters that were
      dropped under outcome 3 with the owner's sign-off.
- [ ] **Step 1: README.** The supported table holds every shipped agent — **twelve** if none was dropped —
      each row carrying the support level and all three capability states (§6.1's binding rule; a support
      column alone is the presentation V3:139-140 forbids). Correct the three pre-existing rows to the
      registry's versions: the table still says Claude 2.1.270 and Codex 0.153.4 while `REAL_ADAPTERS`
      carries 2.1.292 and 0.160.1. Keep the Windows shim paragraph: nine agents are npm-installed, refused
      by default on Windows, and measured only in a Linux container — both facts, plainly.
- [ ] **Step 2: Design §8's correction duty.** Apply the ledger's contradictions to the design's §8 table,
      dated, so the document is not left asserting what the probe disproved (§5.1 item 4).
- [ ] **Step 3: ROADMAP.** SP4 `done`, SP5 `next`; resolve the `ProfilePresence::Known` note — D10 settled
      it: reserved, not dead. Remove the now-closed `probe_candidates` debt entry (Task 1).
- [ ] **Step 4: TODO.** Per design §11: every capability left `Unknown` that an authenticated measurement
      could settle; every adapter registered with `upstream_version: "unknown"`; every adapter dropped under
      outcome 3; Q6 if unanswered; and §6.1's three binding consequences for SP5 (the matrix may not become
      conditional; SP5's JSON must carry capabilities in the same object as the support level; every future
      surface naming a support level must name the capabilities).
- [ ] **Step 5: CONTRIBUTING.** The "Adding an adapter" list already exists; point it at this plan's
      template and at `docs/evidence/README.md` if anything drifted during SP4b.
- [ ] **Step 6: Run the gate.** Expected: all green, including the Evidence job on the pull request.
- [ ] **Step 7: Commit.** `docs: close SP4 — twelve adapters, roadmap and debt updated`.

**Checkpoint: complete**
- [ ] Twelve adapters registered (or the drops are signed off and tracked), each with a `custody: ci`
      transcript whose artifact verified byte-for-byte in the Evidence job.
- [ ] `cargo nextest run --workspace`, `just probe-tests`, `just check` all green on Linux; the Windows and
      macOS CI legs green.
- [ ] The §6.1 rules are intact on every surface the SP4b work touched: no support level without its
      capabilities.

---

## Review Focus

The five things most likely to be wrong in a way a user would feel, each with the task that owns it:

1. **A transcript and its registration land in different commits** — the allow-list or Gate A is red in
   between, or (worse) the transcript is committed "temporarily" and never joined. Owner: every adapter
   task, rule 6, Step 9. The suite is the test: `cargo nextest run -E 'test(gate_a) or test(every_file_in_docs_evidence)'`
   must be green at every commit, never just at the end.
2. **An `upstream_version` that does not match the transcript's filename** — Gate A resolves a path from
   the registry and fails, or (if someone "fixes" it by renaming the file) the Evidence job's byte
   comparison fails against the artifact. Owner: adapter template Step 1 and Step 2. Test:
   `gate_a_every_adapter_has_the_transcript_its_version_resolves_to`.
3. **A credential claim stronger than the evidence** — the standing red flag (§5.1 item 2): no shipped
   adapter may claim `CredentialIsolation: Supported`, and all nine have a documented bypass variable.
   Owner: each adapter task's credential rule. Mechanical part: Gate B's biconditional; content part:
   §5.1 item 1, a reviewer reading the basis against the transcript.
4. **Cline's shape is not expressible and gets improvised anyway** — a speculative two-mechanism helper
   would be a public-enum change made by someone who was not asked. Owner: Task 11's stop rule. The stop
   report `SHAPE_NOT_EXPRESSIBLE` is the behaviour that must happen, not a step to skip.
5. **The known-agents string at `launch.rs:245` is updated with the wrong order** — it renders
   `known_agents()`, which is `REAL_ADAPTERS` order, and every future adapter task depends on the invariant
   "this string matches the registry". Owner: template Step 6. Test: the `unknown_agent_lists_known_agents`
   test itself; if it needs editing to pass, the order was wrong, not the string.

## Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| An artifact expires before its adapter lands (90-day retention) | High — a transcript whose artifact is gone cannot be introduced or modified (§5.3 fails closed) | Tasks 3–11 run immediately after Task 2; the ledger records the window; a re-dispatch re-measures |
| The probe run contradicts §8 for several adapters at once | Medium — shapes change, tasks re-scope | Task 2's correction duty happens before any adapter is written; contradictions are owner-visible in the ledger |
| A vendor ships a breaking change between probe and release | Medium — evidence is a snapshot | Accepted and recorded (§12); `doctor` (SP5) is the drift detector |
| Task 1's source change invalidates the capstone over the shell delta | Medium — process, not code | Task 1 Step 7 records it for the audit round; do not bury it |
| Nine adapter commits each extend the same three files | Low — merge friction inside one branch | Tasks 3–11 are sequential on one branch by design; the registry order is fixed by the plan |
