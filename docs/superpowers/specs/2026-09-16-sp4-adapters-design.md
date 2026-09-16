# SP4 — The remaining nine adapters: design

**Status:** draft. Two panel rounds folded (twelve seats). D6 adjudicated by the owner on 2026-09-16:
follow V3 (§4.1).
**Branch:** `sp4-adapters` (from `main` at `32e8673`).
**Oracle:** `agent-profile-implementation-spec-v3.md` (called "V3" below). Where this document and V3
disagree, V3 wins; report the conflict instead of resolving it silently.
**Previous sub-project:** SP3, `docs/superpowers/specs/2026-09-15-sp3-resolution-design.md` (merged at
`4ea5976`, released as v0.0.2).

## 1. Goal

SP4 delivers V3 §35 phase 4B **less its `doctor` requirement**: the nine remaining adapters — Gemini CLI,
GitHub Copilot CLI, OpenCode, Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI and Amp. It also
closes the gap that makes shipping them honest: today an adapter's support level and capability states are
stored and tested but never shown to a user, so an unproven adapter is indistinguishable from a proven one
at the command line.

**Reported conflict with the oracle.** V3 §35 phase 4B lists five per-adapter requirements
(`agent-profile-implementation-spec-v3.md:1368-1374`), the last of which is "doctor support". `doctor` is
SP5 work and returns `Error::NotYetImplemented` today (`crates/agent-profile/src/cli.rs:188`, reached for
the `"doctor"` command name at `:94`). SP4 therefore completes four of the five; SP5 completes the fifth
for all twelve adapters at once. This is a deliberate, recorded divergence, not an oversight.

SP4 ends with twelve adapters registered, each carrying evidence that a reviewer can check against a
committed transcript, and with the report stating plainly how far each adapter is proven.

## 2. Scope

### 2.1 In scope

| Area | V3 |
|---|---|
| Rendering support level and capability states in the dry-run and `--verbose` report | §3, §26, §37 |
| A launch-path hedge for adapters that are not `Proven` | §3, §37 |
| Evidence invariants enforced by the contract suite (gates A–C, §5) | §28, §37 |
| Probe-harness correctness fixes: exit status, per-command timeout, step order, baseline snapshot (D13) | §34 |
| A behavioural probe step: launch the agent under the wrapper and diff what it wrote | §2 "reverified", §28 |
| Twelve probe scripts — the nine new agents plus re-probes of the three shipped ones | §28 |
| Committed evidence transcripts under `docs/evidence/`, with chain of custody and a retention rule | §28, §37 |
| The Sandbox workflow probing several agents in one dispatch, as a bounded matrix | §34 |
| Nine adapters, each with metadata, contract row and end-to-end row | §2, §3, §21, §22, §28 |
| Deleting `ProfilePresence::Known` and its `status` label | §8 |

### 2.2 Out of scope

- `doctor` (SP5), including mechanism-drift detection against the recorded evidence. See §1.
- JSON output of capabilities and evidence (SP5, V3 §32).
- `create`, `list`, `delete`, `repositories` (SP5).
- **Any authenticated measurement.** No probe logs in to any vendor, and no secret is ever placed in CI.
  SP4 therefore ships no `CredentialIsolation` claim stronger than `Unknown` for the nine. The later
  upgrade path exists and its custody shape is defined (§7.4.1), but performing it is not SP4 work.
- Changing the isolation mechanism of the three shipped adapters. Their metadata gains the provenance
  prefixes of D5, and they gain committed transcripts (§9), and nothing else.
- Native-profile adapters (see D10).

## 3. Why the visibility work belongs to SP4, not SP5

V3 parks caveat disclosure in `doctor` (§37, "doctor exposes important caveats"), and `doctor` is SP5.
Measured against the tree at `32e8673`:

- `report_lines` (`crates/agent-profile/src/output.rs:31-87`) renders `agent`, `profile`, `executable`,
  `repository`, `mechanism`, `environment`, `creates`, `arguments` and `note`. No support level and no
  capability state reaches a user.
- `doctor` returns `Error::NotYetImplemented` (`crates/agent-profile/src/cli.rs:188`), exit 2.
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
| D1 | SP4 runs in two phases with a measurement step between them: **SP4a** (visibility, invariants, probe-harness fixes, twelve probe scripts, workflow) against code that exists now; **the probe run**, twelve agents in CI; **the fold commit**, which lands the transcripts and Gate A's transcript half together; then **SP4b**, the nine adapters. | reasoned: the owner's plan-vs-spec rule forbids a line-level plan whose "existing code" is an unmeasured third-party product. measured: `sandbox/run.sh:51-54` refuses a probe with no script, so every probe script must exist before the probe run — the scripts are SP4a deliverables. The fold commit is a separate step because Gate A's transcript assertion cannot land before the transcripts do: see §5.2. |
| D2 | The report gains a `support:` line and an `isolation:` block naming each capability's state and its `basis`, in both report modes. | measured: the report is the only surface carrying `mechanism` (`output.rs:51`). The `basis` shows in both modes because D3's hedge directs the user to the report for the reason, and a mode that withheld the reason would make that pointer false. |
| D3 | An adapter whose support level is not `Proven` prints one line to stderr before the agent starts, on a real launch — not on a dry run, which already shows the whole block. It names **every** capability that is `Unknown` or `NotSupported`, not the first. A configuration key suppresses it. | measured: the report renders only for `--dry-run` and `--verbose` (`crates/agent-profile/src/cli.rs:596`, `:605`); a plain launch prints nothing. Naming only the first capability would under-report OpenCode, which §8.2 gives two `NotSupported` capabilities, and `Capability::ALL` has a fixed order (`metadata.rs:39-41`) that would always hide the same one. |
| D4 | **Gate A.** No adapter is registered unless its mechanism token — the variable name or the flag spelling — was observed in the agent's own artefact at the version recorded in `evidence.upstream_version`. Enforced mechanically, **over `REAL_ADAPTERS` only**, by: `source_url` is a URL or the literal `measured` with non-empty `notes` (**A-shape**, lands in SP4a); and `docs/evidence/<id>-<upstream_version>.md` exists and contains the adapter's mechanism token (**A-transcript**, lands with the fold commit). | measured: `metadata_invariants` asserts only non-emptiness today (`crates/agent-profile/tests/adapter_contract.rs:264-266`), which any placeholder satisfies; a URL-shape check alone is satisfied by `https://example.com/`. The `REAL_ADAPTERS` scope is load-bearing: `registry()` includes `Fake` in every debug build (`adapter/mod.rs:112-120`) and tests build in debug, so an unscoped Gate A would demand `docs/evidence/fake-0.0.0.md` (`adapter/fake.rs:24`) for a fixture with no upstream product — satisfiable only by hand-writing the exact artefact §7.4 exists to detect. The pinned-support assertion already uses `REAL_ADAPTERS` (`adapter_contract.rs:275`), so the distinction exists in the file. |
| D5 | **Gate B.** Every `basis` begins with `measured: `, `cited: ` or `unmeasured: `, contains no newline, and — for any state other than `NotSupported` — either names a specific bypass or states that a search found none. The prefix states the provenance of the evidence establishing the **state**; where facts of differing provenance support one state, the prefix takes the weakest and per-fact inline markers are kept. **`unmeasured:` and `CapabilityState::Unknown` imply each other**, asserted both ways. | measured: this is what the shipped adapters already do informally — `claude.rs:25-27` enumerates eight override variables, `codex.rs:29-31` four. The biconditional is what makes D7 enforceable: without it, an implementer reaches `Proven` by writing `NotGuaranteed` with an `unmeasured:` basis, and every mechanical gate still passes. The no-newline rule protects the "one `Vec` entry is one line" contract (`output.rs:30`, `cli.rs:596-597`). |
| D6 | **Gate C.** `SupportLevel::Proven` requires non-empty `notes` and **at most one** `Unknown` capability claim. The pinned support list at `adapter_contract.rs:275-286` is **kept** alongside the new invariant. | owner ruling, 2026-09-16, adjudicating the oracle conflict in §4.1: V3:132-133 permits exactly this. measured: the pinned triple and the invariant catch opposite mistakes. Gate C constrains only `Proven ⇒ …` and never asserts that any adapter *is* `Proven`, so replacing the list would let a one-word demotion of `claude` pass the suite silently while adding a permanent stderr banner to the flagship adapter. |
| D7 | A capability that cannot be measured because measuring it needs an authenticated session is `CapabilityState::Unknown` with an `unmeasured:` basis. No enum variant is added. | reasoned: `NotGuaranteed` and `Conditional` are claims about a mechanism that was understood — both existing uses of `NotGuaranteed` describe a known leak path (`claude.rs:44-46`, `aider.rs:38-43`), and `Conditional` promises the conditions are enumerable (`codex.rs:43-45` lists three variables by name). Using either for "we could not log in" borrows the authority of a measurement that never happened. `NotSupported` is a positive negative claim and would defame a product that may isolate correctly. D5's biconditional makes this mechanical rather than advisory. |
| D8 | `sandbox/probes/common.sh` gains `probe_behaviour <agent> <default-location>`: snapshot `<default-location>` after install and before launch, launch the agent under `agent-profile` (not `--dry-run`), then record the delta at both the profile directory and `<default-location>`. It runs **before** any unwrapped invocation of the agent. | measured: today `probe_agent_profile` records `cargo build`, `--version` and a dry run (`sandbox/probes/common.sh:22-26`), so it captures what `agent-profile` *plans*, never what the agent *does*. Ordering is load-bearing: `claude.sh:5-7` runs `claude --version` and `claude --help` natively before `probe_agent_profile`. The baseline snapshot is equally load-bearing and ordering alone does not supply it: the install runs before everything and executes vendor code (§9), so an installer that creates the default location would otherwise be attributed to the wrapped launch, making `ConfigIsolation` read worse than the truth. The baseline itself goes in the transcript so a reviewer sees what was already there. |
| D9 | Probes run in the Actions Sandbox workflow as a **matrix, one job per agent**, bounded by a concurrency group and a deduplicated, length-capped agent list. | measured: only the Podman branch of `sandbox/run.sh` creates a private ephemeral image store (`:84`) and deletes it at cleanup (`:105-108`); the Docker branch does `chmod a+rwx "$out"` and nothing else (`:88`), so agent install layers persist in the host daemon. Podman is absent from the owner's machine; the workflow sets `AGENT_PROFILE_SANDBOX_ENGINE: podman` (`.github/workflows/sandbox.yml:43`) on a runner VM that is discarded. A matrix is required rather than a loop: each invocation builds a fresh image (`sandbox/run.sh:124`) containing a Rust toolchain, so twelve sequential probes would not fit the job's 60-minute cap (`sandbox.yml:35`), and one hanging agent would consume the budget of all the others. The bounds are required because the matrix removes that cap: `timeout-minutes` is a job attribute, so twelve jobs are twelve independent 60-minute budgets, and `sandbox.yml` has no `concurrency` key today while `docs.yml:17-19` and `release-plz.yml:70` show the repository's idiom. |
| D10 | `ProfilePresence::Known` and the `"known"` arm are deleted. | measured: the variant has exactly two occurrences — its declaration (`crates/agent-profile/src/adapter/metadata.rs:102`) and a `status` label (`crates/agent-profile/src/cli.rs:739`); no code constructs it. The default `presence()` can only return `Materialized` or `Absent` (`adapter/mod.rs:48-58`), so `Known` needs an adapter that identifies a profile without owning a directory. All nine mechanisms resolve to a path `agent-profile` itself creates. The repository already declined the native-profile route for Codex, the adapter V3 §2 most entitles to it (`codex.rs:69-75`, and `-p/--profile` is accepted rather than conflicting: `adapter_contract.rs:194`). cited: research on 2026-09-16 found no named-profile concept among the nine. Reversal costs one variant and one match arm. |
| D11 | An SP4 adapter ships with `conflicts: &[]` unless a probe transcript shows the option in that agent's `--help` at the pinned version. | reasoned: V3 §21 ("If the wrapper cannot prove a conflict, it must not guess"). An empty list is already a legal contract row and both `claude` and `codex` use it (`adapter_contract.rs:194-195`). Aider's list is a measured prefix-abbreviation set (`aider.rs:56-59`), which is what a proven conflict list costs; nine of those cannot come from documentation. |
| D12 | All nine ship in one wave rather than split by measurability. | cited: research on 2026-09-16 established that every one of the nine installs and runs `--help`/`--version` with no account, and every one needs an account to do real work. The proposed split axis puts all nine in the same bucket, so it separates nothing. |
| D13 | The probe harness is repaired **before** the probe run: `probe_record` propagates failure, every recorded command runs under a timeout, `cargo build` runs once per job rather than once per recorded step and its failure is recorded, and `probe_behaviour` takes a baseline snapshot. | measured, and this is the finding that would have wasted the whole run: `probe_record` ends with `set -e` (`sandbox/probes/common.sh:18`), and a shell function returns the status of its last command, so `probe_record` **always returns 0**. Every probe script ends in `probe_agent_profile`, which ends in `probe_record dry-run`, so a probe whose install failed and whose every later command returned 127 still exits 0 and the Actions job is green. There is no `timeout` anywhere in `common.sh` or `run.sh`, and `cargo build` at `common.sh:23` is unguarded, so under `set -eu` (`:7`) a build failure aborts the probe after the install, leaving transcripts that look complete with no behavioural step. Note the scope: under D9's matrix the crate is built once per *job*, which is once per agent — building it once across all agents is not achievable and is not claimed. |

### 4.1 D6: the oracle conflict, and the owner's ruling

The draft rule was "`Proven` requires that no capability claim is `Unknown`". V3 states the opposite
permission directly:

> `agent-profile-implementation-spec-v3.md:132-133` — "An adapter may be `Proven` while one capability
> remains `Conditional`, `NotGuaranteed`, or `Unknown`."

This was material: under V3's rule an adapter whose config and state isolation are measured and whose
credential isolation alone is `Unknown` may be `Proven` — the exact shape §6 predicts for most of the
nine. Under the draft rule all nine would have been `Experimental`, each carrying D3's stderr hedge on
every launch forever.

**Owner ruling, 2026-09-16: follow V3.** `Proven` tolerates at most one `Unknown` capability. The support
level keeps meaning "the mechanism is proven"; the capability matrix carries the nuance, and §7.1 makes
that matrix visible, which is what stops `proven` from overclaiming. A second `Unknown` capability means
the mechanism itself is not understood, and that is what `Experimental` is for.

## 5. The evidence bar, as contract-suite invariants

All gates are assertions in `metadata_invariants`
(`crates/agent-profile/tests/adapter_contract.rs:241`). **Gates A and C iterate `REAL_ADAPTERS`, not
`registry()`** — see D4 — while Gate B applies to every adapter including `fake`, which has real claims
and must not model bad ones.

```text
Gate A   source_url is a URL, or the literal "measured" with non-empty notes        [SP4a]
         AND docs/evidence/<id>-<upstream_version>.md exists and contains the
             mechanism token                                                        [fold commit, §5.2]
         AND every file in docs/evidence/ is referenced by exactly one adapter      [fold commit]
Gate B   every basis starts with "measured: ", "cited: " or "unmeasured: "
         AND contains no newline
         AND starts with "unmeasured: " if and only if its state is Unknown
Gate C   support == Proven  =>  notes is non-empty
                            AND at most one capability claim is Unknown
         AND the pinned support list still holds for claude, codex and aider
```

`upstream_version` is constrained to `[A-Za-z0-9._-]+` so that `<id>-<version>.md` always names one
unambiguous file: it is an unconstrained `&'static str` today (`metadata.rs:20`) and Gate A now builds a
path from it.

Gate A's third clause is the converse of its second, and it exists because without it the evidence
directory and the registry drift apart silently — an orphaned transcript asserts a measurement about a
version nobody supports and looks, at a glance, exactly like current evidence.

### 5.1 The manual surface

These rules cannot be asserted, and are named here together so no one mistakes them for gates:

1. **Gate B's content half** — whether a `basis` sentence describes a real bypass. No test can judge a
   sentence; D8's transcript is what a reviewer checks it against.
2. **The standing red flag** — no shipped adapter claims `CredentialIsolation: Supported`: Claude is
   `Conditional` (`claude.rs:38`), Codex `Conditional` (`codex.rs:42`), Aider `NotSupported`
   (`aider.rs:46`). Every one of the nine has a documented API-key or token variable that bypasses its
   mechanism (§8). An SP4 adapter arriving as `Supported` means the measurement is absent or wrong.
3. **Transcript fidelity** — Gate A proves a file exists and contains a token; only a human can tell a real
   transcript from a plausible hand-written one. §7.4's chain of custody is what makes that check possible.
4. **§8's correction duty** — "where a probe contradicts a row, the probe wins" requires someone to re-read
   §8 after the run. Nothing enforces it.
5. **Mutant proof of new tests** (§10) — `just mutants` exists (`justfile:68-69`) but no CI job or hook
   runs it.

### 5.2 Why Gate A is split across two commits

`docs/evidence/` does not exist in the tree today (`docs/` holds `superpowers/`, two ledgers and
`dev-tooling.md`), and the three shipped adapters name versions — `claude` `2.1.270` (`claude.rs:23`),
`codex` `0.153.4` (`codex.rs:27`), `aider` `0.86.2` (`aider.rs:32`) — whose transcripts were never
committed. A Gate A that asserted transcript existence inside SP4a would therefore turn the branch red for
three adapters, and the only artefact that could make it green is produced by the probe run that D1
sequences *after* SP4a.

So Gate A lands in two pieces: the shape half in SP4a, the transcript half in the fold commit that also
adds the transcripts. Neither commit is ever red. This is the one place where an SP4a deliverable is not
buildable against code that exists now, and splitting it is what preserves D1's premise rather than
quietly breaking it.

## 6. What can and cannot be measured

The probe of D8 runs unauthenticated, which bounds each capability:

| Capability | Measurable unauthenticated? | How |
|---|---|---|
| `ConfigIsolation` | yes | set the mechanism, launch, diff against the baseline: the agent's config lands under the profile directory rather than the default location. |
| `StateIsolation` | partly | history, session and log files appear in the delta only if the agent writes them before it needs the network. |
| `CredentialIsolation` | **no** | proving that stored credentials separate requires logging in to each vendor. |

Consequently `CredentialIsolation` for all nine is `Unknown` with an `unmeasured:` basis naming the
vendor session required, unless a probe shows a credential file created without authentication. Under D6
that single `Unknown` does not by itself prevent `Proven`: an adapter whose config isolation is measured
and whose state isolation is measured or honestly `NotSupported` can ship `Proven` with its credential
caveat visible in the matrix. An adapter that also cannot demonstrate config isolation has two `Unknown`
claims and is `Experimental`, which is the state meaning "the mechanism itself is not established".

An adapter may raise `CredentialIsolation` later without new code: it is a metadata edit plus a new
transcript. §7.4.1 defines the custody shape such a transcript must carry, because it cannot be produced
in CI. `doctor` (SP5) is where drift against the recorded evidence is detected.

## 7. Architecture

### 7.1 Report rendering (D2)

`report_lines` needs adapter metadata, which it cannot currently reach: its signature is
`report_lines(&PlannedLaunch, &Resolution, ReportMode)` (`output.rs:31-35`) and `PlannedLaunch`
(`adapter/mod.rs:96-109`) carries no metadata. **The chosen plumbing is a fourth parameter,
`metadata: &'static AdapterMetadata`**, passed by both call sites (`cli.rs:596`, `:605`). It is not a new
`PlannedLaunch` field, because `PlannedLaunch` is what an adapter *returns from planning* and metadata is
static; and it is not an internal `adapter::lookup`, because that would let the report describe a
different adapter from the one that planned the launch.

Two labels follow `mechanism`, using the existing `line`/`continuation` helpers and `LABEL_WIDTH`
(`output.rs:19`):

```text
support:      proven
isolation:    config: supported
                measured: settings.json and history move with the variable
              credentials: unknown
                unmeasured: requires an authenticated vendor session
              state: not guaranteed
                measured: session files stay in the default location
```

Capability order is `Capability::ALL` (`metadata.rs:39-41`), which is fixed, so the rendering is
deterministic without sorting. State spellings are lower-case with spaces: `supported`, `not supported`,
`not guaranteed`, `conditional`, `unknown`. The `basis` line is indented one level past the capability and
is rendered in both modes.

**A capability with no claim renders as `not declared`, with no `basis` line.** The renderer must not
assume a claim exists for every capability: that invariant is asserted only over the registry
(`adapter_contract.rs:252-261`), while `report_lines` is reachable from outside it — `SECRETIVE`
(`adapter/mod.rs:493-508`) declares `capabilities: &[]` and its plan is passed to `report_lines` at
`adapter/mod.rs:550`. A lookup-and-unwrap would panic there; a silent skip would make "not claimed"
indistinguishable from "not rendered". §10 pins the behaviour with a test.

**This insertion invalidates every positional assertion over the report.** Known today:
`output.rs:392-393` (`existing[5]`, `existing[6]`), `output.rs:400` (`lines.len() == 7`) and
`tests/launch.rs:143` (`report.len() == 7`). SP4a re-baselines each of them; §10 makes that a task rather
than a surprise.

### 7.2 The launch hedge (D3)

When `metadata().support` is not `Proven`, and the launch is real rather than a dry run, the CLI writes
one line to stderr before the launcher runs:

```text
agent-profile: kiro is experimental: config not guaranteed, credentials unknown. Run with --dry-run for detail.
```

The capability names are exactly the report's — `config`, `credentials`, `state` (§7.1) — so one
vocabulary covers both surfaces. Every capability that is `Unknown` or `NotSupported` is listed, in
`Capability::ALL` order; if none is, the line names the support level alone. A configuration key,
`hide_support_warning = true`, suppresses it; without an opt-out, a warning that can never be cleared is
trained away within a week.

Under D6's ruling this line is rare rather than universal — it marks an adapter whose mechanism is not
established, not merely one with an unmeasured credential claim. That is what keeps it readable. Its
limits are recorded in §12 rather than papered over.

### 7.3 Probe harness (D8, D13)

`common.sh` is repaired first:

- `probe_record` returns the recorded status and accumulates failures, so a probe that failed exits
  non-zero. This is D13's core fix; without it the probe run's green light carries no information.
- every recorded command runs under `timeout <n>`, and a timeout status is recorded as the "cannot run
  non-interactively" fact rather than being left to the job cap.
- `cargo build` runs once per job, not once per recorded step, and its failure is recorded rather than
  aborting the script silently.

`probe_behaviour <agent> <default-location>` then: snapshots `<default-location>` after the install and
before any launch; runs the agent under `agent-profile` with a throwaway application root and a
non-interactive argument; and records the delta at the profile directory and at `<default-location>`. The
default location is a per-agent fact, so each probe script passes it; `common.sh` cannot derive it.
`probe_behaviour` runs **before** any unwrapped invocation of the agent.

The baseline is recorded in the transcript, not just used to compute the delta: the install ran vendor
code before anything else, and a reviewer needs to see what that code had already created.

A probe never supplies a credential and never waits for input.

### 7.4 Evidence transcripts (D8)

`docs/evidence/<agent>-<version>.md` holds, for one probe run: the install command including the resolved
version, `--version` output, the `--help` excerpt containing the mechanism token, the strings excerpt
listing candidate bypass variables, the pre-launch baseline, the behavioural delta, and a **chain of
custody** — the workflow run URL, the harness commit, and every recorded step's exit code verbatim.
Without the last part a plausible transcript can be hand-written in ninety seconds and no gate, human or
mechanical, could tell.

Captured output is untrusted vendor text. Before it becomes a committed file it is stripped of terminal
control sequences and bounded in size; an excerpt that must be truncated says so at the truncation point.

**Retention.** One transcript per adapter is live at a time: the one Gate A resolves from the current
`upstream_version`. A re-measurement replaces the file it supersedes — git history holds the old one, and
Gate A's third clause fails if an orphan is left behind. An `unknown` transcript from a failed probe
(§9 outcome 2) is deleted when the agent is later probed successfully; leaving it would be a durable false
statement that the agent cannot be installed.

**Probe scripts take an optional version argument.** The default resolves to whatever the registry serves,
which is how a version is discovered; passing the recorded version is how a measurement is repeated.
Today's probes pin nothing (`sandbox/probes/claude.sh:4`, `aider.sh:5`), and CONTRIBUTING's pinning
guidance covers the *interpreter*, not the package (`CONTRIBUTING.md:108-109`).

#### 7.4.1 Off-CI custody, for authenticated measurement

A transcript produced outside CI has no workflow run URL, so it carries a different, explicitly weaker
custody block: the sandbox and its version, the host platform, the date, and who performed it. It is
marked `custody: off-ci` on its first line so a reviewer never mistakes it for a CI transcript.

This shape exists because §6's upgrade path would otherwise be unreachable: an authenticated measurement
cannot run in CI (§2.2), so the one transcript that could ever justify raising `CredentialIsolation` is
the one that cannot carry CI custody. The measurement still happens in a disposable sandbox — SP2 used
Sandboxie-Plus for exactly this (SP2 design D9) — never on the host. Performing such a measurement is out
of scope for SP4; defining how it must be recorded is not.

### 7.5 Workflow (D9)

The `agent` input is renamed to `agents` and accepts a whitespace- or comma-separated list, or `all`. A
`prepare` job parses it, validates each name against `[a-z0-9-]` and against the existence of
`sandbox/probes/<name>.sh`, **deduplicates**, caps the list length, and emits a JSON array. An empty or
unknown name fails the job loudly; a naive split-and-loop would iterate zero times and report success
having probed nothing, and a duplicated name would double the runner cost while colliding on the artifact
name the matrix is supposed to keep unique.

A `probe` job then runs `strategy: { fail-fast: false, matrix: { agent: <that array> } }`, so one agent's
failure or hang cannot consume another's budget, and each job keeps its own 60-minute cap. A
`concurrency` group prevents a re-dispatch from running twelve more installer jobs alongside twelve
already in flight; `docs.yml:17-19` is the repository's idiom for this.

Each job uploads two artifacts: `sandbox-transcript-<agent>`, holding only the curated files that become
the transcript, and `sandbox-logs-<agent>`, holding the raw `build.log`, `output.log` and `diff.txt`. They
are separate because the raw upload is the whole container delta (`sandbox/run.sh:154`) plus the image
build log including toolchain downloads, and a human reviewing twelve of those is the load-bearing step of
§5.1 — burying the reviewable artefact in megabytes of build noise is how that step stops happening.

The `pull_request` trigger keeps running `test` mode only, and no secret is added.

## 8. The nine adapters

Every row below is **cited** — established on 2026-09-16 from first-party documentation or repository
source — and none is yet **measured**. The probe run of §9 converts these into evidence; where a probe
contradicts a row, the probe wins and this table is corrected rather than the adapter bent to fit it.

| Agent | id | Executable | Install | Mechanism (claimed) | Shape | Documented credential bypass |
|---|---|---|---|---|---|---|
| Gemini CLI | `gemini` | `gemini` | npm | `GEMINI_CLI_HOME` | env dir | `GEMINI_API_KEY`, `GOOGLE_API_KEY`, `GOOGLE_APPLICATION_CREDENTIALS` |
| GitHub Copilot CLI | `copilot` | `copilot` | npm | `COPILOT_HOME` | env dir | `COPILOT_GITHUB_TOKEN`, `GH_TOKEN`, `GITHUB_TOKEN` |
| OpenCode | `opencode` | `opencode` | npm or install script | `OPENCODE_CONFIG_DIR`, and `OPENCODE_CONFIG` for a file | env dir | provider API keys; credentials live outside the config directory (§8.2) |
| Cline CLI | `cline` | `cline` | npm | `--config` and `--data-dir`, or `CLINE_DATA_DIR` | undecided (§8.3) | `ANTHROPIC_API_KEY` |
| Pi | `pi` | `pi` | npm or install script | `PI_CODING_AGENT_DIR` | env dir | provider keys, unless `auth.json` is present |
| Kiro CLI | `kiro` | `kiro-cli` | install script or `.deb` | `KIRO_HOME` | env dir | `KIRO_API_KEY` |
| Cursor Agent CLI | `cursor` | `cursor-agent` | install script | `CURSOR_CONFIG_DIR` | env dir | `CURSOR_API_KEY` |
| Continue CLI | `continue` | `cn` | npm or install script | `--config <file>` | config file arg | `CONTINUE_API_KEY`; secrets live in a separate `.env` |
| Amp | `amp` | `amp` | npm or install script | `--settings-file <file>` | config file arg | `AMP_API_KEY`; workspace and managed settings override |

Agent ids are what a user types (`agent-profile gemini work`) and must satisfy `AgentId::parse`
(`adapter_contract.rs:247-250`). Executable names are the binary the adapter discovers, and differ from
the id for Kiro, Cursor and Continue. The install column is cited, not measured, and the probe run tests
it: an install command is itself an unverified claim until a probe executes it.

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
exceeds what an adapter may claim to control.

V3 §2 names two mechanisms for OpenCode (`agent-profile-implementation-spec-v3.md:99`:
"`OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG`") and describes the result as "Config/home isolation". Two
conflicts with the evidence are reported rather than resolved here: the second variable names a config
*file* rather than a home, so it is an alternative to the first and not an addition to it; and "home"
overstates what either variable covers. If the probe finds no OpenCode-specific data-directory variable,
the adapter sets `OPENCODE_CONFIG_DIR` alone and declares `CredentialIsolation` and `StateIsolation`
`NotSupported`, each with a basis naming the directory that does not move. Note the consequence under D6:
two non-`Unknown` negative claims do not block `Proven`, so OpenCode can be `Proven` *and* honestly state
that it isolates configuration only. The owner is asked to confirm that reading of V3 §2 when approving
this spec.

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
only because it was measured (`aider.rs:17-18`, SP2 design D5). The same question is open for Continue's
`config.yaml` and Amp's `settings.json`, and the answer must be the smallest content the agent accepts
that sets no option — `agent-profile` never invents configuration (V3 §9). Until a probe establishes it,
no contents are written into the spec.

### 8.5 Kiro's mechanism is documented as unreliable

`KIRO_HOME` is documented, but an open upstream report describes subsystems ignoring it and using
`~/.kiro` regardless, failing silently. If the probe reproduces that, the adapter ships with
`ConfigIsolation: NotGuaranteed` and a basis citing the specific subsystems, which is exactly the state
`NotGuaranteed` exists for; combined with an `Unknown` credential claim that is one `Unknown`, so D6 still
permits `Proven` — the matrix, not the support level, is what tells the user the mechanism leaks. Kiro CLI
is the rebrand of Amazon Q Developer CLI; the executable is `kiro-cli`.

## 9. The probe run

Between SP4a and SP4b, one dispatch of the Sandbox workflow probes **twelve** agents: the nine new ones
and re-probes of `claude`, `codex` and `aider`. The three shipped adapters are included because Gate A
requires a committed transcript and theirs were never committed — their SP2 evidence lives in gitignored
`target/sandbox`, which means the three adapters carrying the strongest claims in the repository are the
only three with no checkable evidence. SP4 inverts that rather than grandfathering it.

**The commit is manual and human-performed.** A maintainer downloads the artifacts, reviews each
transcript, and commits them together with Gate A's transcript half (§5.2). The probe job keeps
`permissions: contents: read` (`.github/workflows/sandbox.yml:28-29`) and **must never be granted write
access**: it runs `npm install --global` and vendor install scripts for twelve third-party packages, any
of which can execute arbitrary code in a postinstall hook. A repository-write credential in that job would
let one compromised package push to the default branch.

Four outcomes, and every probe lands in exactly one:

1. **Mechanism confirmed** — the token appears at the resolved version and the behavioural delta shows the
   files moved. The adapter ships with measured bases.
2. **Probe failed** — the agent will not install, or will not run non-interactively. The adapter is still
   written, from cited evidence, with `ConfigIsolation: Unknown`. That is a second `Unknown` alongside
   credentials, so D6 makes it `Experimental`, which is correct: nothing about its mechanism was
   established. Its evidence entry records `upstream_version: "unknown"` and a
   `docs/evidence/<agent>-unknown.md` transcript recording the failure, so Gate A still resolves to a real
   file and no version string is invented. §7.4's retention rule removes that file when the agent is later
   probed successfully.
3. **Mechanism disproved** — the agent installs and runs, and the claimed token is absent from its
   artefacts. Gate A then forbids registration. **The adapter is not registered in SP4**; its row moves to
   `TODO.md` as tracked debt with the transcript as evidence. This is an explicit exception to "no adapter
   is dropped", which covers outcome 2 only, and it is the most valuable result a probe can produce.
4. **Mechanism differs** — the agent exposes a *different* mechanism from the claimed one. §8's table is
   corrected and the adapter is written against what was measured.

No probe failure is worked around by installing the agent on the owner's host. A disposable local sandbox
remains available for a later authenticated measurement under §7.4.1.

## 10. Testing (V3 §34)

| Suite | Addition |
|---|---|
| `tests/adapter_contract.rs` | gates A-shape, B and C in `metadata_invariants` (SP4a); Gate A's transcript clauses in the fold commit (§5.2); one `expected()` row per new adapter (the `panic!` at `:131` makes a missing row a failure, not a silent pass); one `conflict_contract` row each. |
| `tests/adapters_e2e.rs` | one row per new adapter, launching the renamed `fake-agent`, proving the plan reaches the child. No real agent runs in CI. |
| `src/output.rs` unit tests | the `support:` and `isolation:` rendering for each `CapabilityState` spelling; the `basis` line in both modes; **a capability with no claim renders `not declared` and does not panic** (§7.1). **Re-baseline the positional assertions at `output.rs:392-393` and `:400`.** |
| `tests/launch.rs` | the D3 stderr hedge: present for an `Experimental` adapter, absent for a `Proven` one, absent on a dry run, listing *every* leaking capability rather than the first, suppressed by the configuration key, and on stderr rather than stdout. **Re-baseline `launch.rs:143`.** |
| Probe harness | `probe_record` returns non-zero for a failed command — the D13 regression that would otherwise make the whole probe run meaningless. Testable without any agent: record `sh -c 'exit 3'`. |
| Deletion of `Known` | the `status` presence labels stay covered by the existing SP3 resolution tests; no test may assert the string `known` afterwards. |

Every new test is proven non-vacuous with a logic mutant, per the repository's assertion-strength
discipline: a test that cannot fail is a test that certifies nothing.

## 11. Documentation

- `README.md`: the supported-agent table grows to twelve, with a support-level column, and states the
  Windows shim consequence of §12.
- `CONTRIBUTING.md`: how to add an adapter, pointing at the gates and at `docs/evidence/`.
- `docs/evidence/README.md`: what a transcript is for, what chain of custody it must carry (both shapes,
  §7.4 and §7.4.1), how to refresh one, and that a refresh replaces rather than accumulates.
- `ROADMAP.md`: SP4 done, SP5 next; the `ProfilePresence::Known` note added at `8741ea0` is resolved.
- `TODO.md`: any capability left `Unknown` that an authenticated measurement could settle is tracked debt,
  as is any adapter dropped under outcome 3 of §9.

## 12. Known limits

- **Credential isolation is unproven for all nine.** SP4 states this rather than fixing it; fixing it
  needs a paid account per vendor. Under D6 an adapter may still be `Proven` with that single `Unknown`,
  which is why §7.1's matrix is mandatory rather than optional: the support level alone would overclaim.
- **Evidence is a snapshot.** A transcript pins one version on one day. Drift detection is `doctor` (SP5),
  and until then a mechanism can change upstream without the repository noticing.
- **The behavioural probe proves where files landed, not that nothing leaked.** A delta shows the config
  directory moved; it cannot show that no credential was read from a shared location.
- **`StateIsolation` measurement is partial** for agents that write history only after a network call.
- **The hedge can be erased by the agent it warns about.** It is written to stderr immediately before
  `exec` hands the terminal over, and an agent that opens a full-screen alternate-screen TUI may clear it
  before it is read. The durable disclosures are the report and the README; a first-launch acknowledgement
  belongs with `create` in SP5.
- **On Windows, most of the nine are both unmeasured and refused by default.** They are npm-installed, so
  the executable is a shim, and a shim is refused with a hint rather than parsed (`src/exe.rs:32`, SP2
  design D2, V3 §23.2) — a user must set `executable` in the configuration. Meanwhile the probe runs in a
  Linux container, so the evidence behind their claims is Linux evidence. SP2 measured Windows binaries
  through Sandboxie (SP2 design D9); SP4 does not, because twelve agents through a manual Windows sandbox
  is not a repeatable gate. These two facts compound and the README must say so plainly.
- **Twelve adapters share one probe run.** If the harness changes between SP4a and the run, transcripts and
  harness can disagree; the chain of custody records the harness commit.
- **A matrix dispatch costs up to twelve times one job's budget.** The concurrency group bounds
  simultaneous dispatches, not the cost of a single one.

## 13. Open questions, and where each is resolved

| # | Question | Resolved by |
|---|---|---|
| Q1 | Does `CLINE_DATA_DIR` isolate state without enabling sandbox mode? | probe run (§9); selects the shape in §8.3 |
| Q2 | Does an OpenCode-specific data directory variable exist, and what does `OPENCODE_CONFIG` select? | probe run; decides §8.2's capability states |
| Q3 | Does `KIRO_HOME` hold for all subsystems at the probed version? | probe run; decides Kiro's `ConfigIsolation` |
| Q4 | Do Cursor's stored credentials live under `CURSOR_CONFIG_DIR`? | probe run if observable unauthenticated; otherwise `Unknown` |
| Q5 | Does `--settings-file` affect Amp's credential or session storage? | probe run; otherwise `Unknown` |
| Q6 | Which agents expose a conflicting option in `--help` at the pinned version? | probe transcripts; D11 defaults to `&[]` |
| Q7 | Does any of the nine have a native named-profile concept? | probe run; if one does, D10's deletion is reversed in SP4b |
| Q8 | What is the smallest file content Continue and Amp each accept that sets no option? | probe run (§8.4); no contents are guessed |
| Q9 | Does `Proven` tolerate one `Unknown` capability, per V3, or not? | **answered**: owner ruling 2026-09-16, follow V3 (§4.1) |

No question in this table is answered by reasoning in SP4b's plan. Each is answered by a transcript or
recorded as unanswered, and an unanswered question produces `Unknown`, never a guess.

## Stand-downs

- `DISCARDED-BELOW-FLOOR`: whether the GitHub runner ships Podman for `AGENT_PROFILE_SANDBOX_ENGINE: podman`
  (`.github/workflows/sandbox.yml:43`) — fails closed at `sandbox/run.sh:64`, which exits 2 with a clear
  message rather than proceeding unsafely.
- `DISCARDED-BELOW-FLOOR`: an embedded newline in a hand-written `basis` literal — now asserted by Gate B
  (§5), so the remaining risk is zero rather than merely low.
- `DISCARDED-BELOW-FLOOR`: long `basis` strings wrapping in a narrow terminal — display only; no contract
  claims a maximum width.
- `REJECTED`: a claim that a real adapter could reach `registry()` while omitted from `REAL_ADAPTERS`,
  making the two drift. `adapter/mod.rs:113-120` builds `registry()` from `REAL_ADAPTERS.to_vec()` and
  pushes only `Fake`, so that state is unconstructible.
