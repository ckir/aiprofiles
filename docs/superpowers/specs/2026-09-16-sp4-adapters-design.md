# SP4 — The remaining nine adapters: design

**Status:** draft. Three panel rounds folded (fifteen seats). Owner rulings on 2026-09-16: D6 follows V3
(§4.1); `ProfilePresence::Known` is retained (D10); tier-B adapters are governed by the visible capability
matrix (§1.1, row §3).
**Branch:** `sp4-adapters` (from `main` at `32e8673`).
**Oracle:** `agent-profile-implementation-spec-v3.md` (called "V3" below). Where this document and V3
disagree, V3 wins; report the conflict instead of resolving it silently. Every known divergence is listed
in §1.1 — that table is the discharge of this rule, and a divergence discovered later belongs in it.
**Previous sub-project:** SP3, `docs/superpowers/specs/2026-09-15-sp3-resolution-design.md` (merged at
`4ea5976`, released as v0.0.2).

## 1. Goal

SP4 delivers V3 §35 phase 4B less its `doctor` requirement: the nine remaining adapters — Gemini CLI,
GitHub Copilot CLI, OpenCode, Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI and Amp. It also
closes the gap that makes shipping them honest: today an adapter's support level and capability states are
stored and tested but never shown to a user, so an unproven adapter is indistinguishable from a proven one
at the command line.

SP4 ends with twelve adapters registered, each carrying evidence that a reviewer can check against a
committed transcript bound to the CI run that produced it, and with the report stating plainly how far
each adapter is proven.

### 1.1 Reported divergences from V3

| V3 | What V3 requires | What SP4 does | Disposition |
|---|---|---|---|
| §35, `:1374` | "doctor support" among each adapter's five phase-4B requirements | defers `doctor` to SP5 for all twelve at once; it returns `Error::NotYetImplemented` today (`crates/agent-profile/src/cli.rs:188`, reached at `:94`) | deliberate; SP4 completes four of the five |
| §2, `:99` | OpenCode: "`OPENCODE_CONFIG_DIR` / `OPENCODE_CONFIG`", "Config/home isolation" | one variable, and config only — the second names a config *file*, an alternative rather than an addition, and "home" overstates what either covers (§8.2) | reported; probe confirms (Q2) |
| §2, `:100` | Cline: "`--config`, `--data-dir`" | may ship `CLINE_DATA_DIR` instead, which V3's table does not name, if the probe shows the flags carry a behavioural side effect (§8.3) | reported; probe decides (Q1). V3:112-113 forbids implementing *undocumented* mechanisms; `CLINE_DATA_DIR` is documented, which is what makes this permissible |
| §2, `:91`; §37, `:1464` | "V0.1 implements the same 12 evidence-backed adapters"; "all 12 adapters have evidence" | §9 outcome 3 registers *fewer* than twelve if a probe disproves a mechanism | reported; **outcome 3 requires owner sign-off**, because it reduces V0.1's scope and leaves V3 §37's adapter checkbox unchecked |
| §2, `:110` | "Evidence must be reverified against the supported upstream version before release" | §9 outcome 2 registers an adapter with `upstream_version: "unknown"`, which has no upstream version to reverify against | reported; such an adapter is `Experimental` and is tracked debt in `TODO.md` until a probe succeeds |
| §3, `:139-140` | "Tier-B agents such as Aider, Amp, Continue, and Cursor must not be presented as guaranteed isolated accounts unless evidence proves that claim" | Amp, Continue and Cursor may reach `support: proven` on a config-isolation measurement, with `credentials: unknown` in the matrix | **owner ruling, 2026-09-16**: §7.1's matrix satisfies this. `proven` describes the *mechanism*; the report never shows it without the capability states beneath it, so no user is presented with a guaranteed isolated account. See §6.1 |
| §3, `:132-133` | "An adapter may be `Proven` while **one** capability remains `Conditional`, `NotGuaranteed`, or `Unknown`" | D6 counts only `Unknown`, so an adapter may be `Proven` with one `Unknown` *plus* any number of `Conditional`, `NotGuaranteed` or `NotSupported` claims — which §8.5 relies on for Kiro and §8.2 for OpenCode | reported. The strictest reading — at most one capability outside `Supported` — is already violated by the shipped `claude`, which is `Proven` with `CredentialIsolation: Conditional` (`claude.rs:38`) *and* `StateIsolation: NotGuaranteed` (`claude.rs:44`), and `NotSupported` does not appear in V3's list at all. So the sentence reads as a permission, not a cap. The counting rule is recorded here rather than resolved silently, and it is separate from the Q9 ruling, which asked only about `Unknown` |
| §8, `:346-374` | `ProfilePresence::Known`; "A physical directory is not universally required"; native-profile adapters may represent existence through the native mechanism | **owner ruling, 2026-09-16**: the variant is retained (D10) | no divergence |

## 2. Scope

### 2.1 In scope

| Area | V3 |
|---|---|
| Rendering support level and capability states in the dry-run and `--verbose` report | §3, §26, §37 |
| A launch-path hedge for adapters that are not `Proven` | §3, §37 |
| Evidence invariants enforced by the contract suite (gates A–C, §5) | §28, §37 |
| Binding each committed transcript to the CI run that produced it (§5.3) | §28 |
| Probe-harness correctness fixes: exit status, per-command timeout, step order, baseline snapshot (D13) | §34 |
| A behavioural probe step: launch the agent under the wrapper and diff what it wrote | §2 "reverified", §28 |
| Twelve probe scripts — the nine new agents plus re-probes of the three shipped ones | §28 |
| Committed evidence transcripts under `docs/evidence/`, with chain of custody and a retention rule | §28, §37 |
| The Sandbox workflow probing several agents in one dispatch, as a bounded matrix | §34 |
| Nine adapters, each with metadata, contract row and end-to-end row | §2, §3, §21, §22, §28 |
| Documenting `ProfilePresence::Known` as reserved rather than dead | §8 |

### 2.2 Out of scope

- `doctor` (SP5), including mechanism-drift detection against the recorded evidence. See §1.1.
- JSON output of capabilities and evidence (SP5, V3 §32).
- `create`, `list`, `delete`, `repositories` (SP5).
- **Any authenticated measurement.** No probe logs in to any vendor, and no secret is ever placed in CI.
  SP4 therefore ships no `CredentialIsolation` claim stronger than `Unknown` for the nine. The later
  upgrade path exists and its custody shape is defined (§7.4.1), but performing it is not SP4 work.
- Changing the isolation mechanism of the three shipped adapters.
- Native-profile adapters, and therefore any constructor for `ProfilePresence::Known` (D10).

**What the D5 retrofit actually touches.** Gate B applies to every adapter in the registry, so the
provenance-prefix retrofit reaches `fake` as well as the three shipped adapters: its three bases are the
literal `"test fixture"` with state `Unknown` (`crates/agent-profile/src/adapter/fake.rs:28-43`), which the
biconditional requires to become `unmeasured: `-prefixed. Gate B's *assertable* clauses may also force one
reword: `codex`'s `ConfigIsolation` basis (`crates/agent-profile/src/adapter/codex.rs:38-39`) names no
bypass and does not say a search found none. That content rule is manual (§5.1), so the reword is a review
outcome rather than a test failure — but SP4a must not claim the shipped adapters are untouched.

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
visible rather than merely typed. Under §1.1's tier-B ruling the matrix is not a convenience — it is the
mechanism by which `proven` stays honest, so it renders unconditionally.

## 4. Decisions and their evidence

Evidence marks: **measured** (run and observed; for this document, in the repository at `32e8673`),
**cited** (a first-party page fetched on 2026-09-16), **reasoned**.

| # | Decision | Evidence |
|---|---|---|
| D1 | SP4 runs in two phases with a measurement step between them: **SP4a** (visibility, invariants, probe-harness fixes, twelve probe scripts, workflow, transcript verification) against code that exists now; **the probe run**, twelve agents in CI; **the fold commit**, which lands the transcripts and Gate A's transcript half together; then **SP4b**, the nine adapters. | reasoned: the owner's plan-vs-spec rule forbids a line-level plan whose "existing code" is an unmeasured third-party product. measured: `sandbox/run.sh:51-54` refuses a probe with no script, so every probe script must exist before the probe run — the scripts are SP4a deliverables. The fold commit is separate because Gate A's transcript assertion cannot land before the transcripts do: see §5.2. |
| D2 | The report gains a `support:` line and an `isolation:` block naming each capability's state and its `basis`, in both report modes, unconditionally. | measured: the report is the only surface carrying `mechanism` (`output.rs:51`). Unconditional because §1.1's tier-B ruling rests on the matrix always accompanying the support level. The `basis` shows in both modes because D3's hedge directs the user to the report for the reason. |
| D3 | An adapter whose support level is not `Proven` prints one line to stderr before the agent starts — but only when no report is rendered, since `--dry-run` and `--verbose` already show the whole block. It names **every** capability whose state is not `Supported`. **No suppression key ships in SP4** (§7.2). | measured: the report renders only for `--dry-run` and `--verbose` (`crates/agent-profile/src/cli.rs:596`, `:605`), and the verbose report already goes to stderr on a real launch (`cli.rs:603-608`), so printing both would tell a user who just read the matrix to run `--dry-run` for detail. Listing only `Unknown` and `NotSupported` would have omitted `NotGuaranteed`, which is exactly Kiro's predicted `ConfigIsolation` state (§8.5) — the earlier draft's own example contradicted its own rule. `Capability::ALL` has a fixed order (`metadata.rs:39-41`). |
| D4 | **Gate A.** No adapter is registered unless its mechanism token was observed in the agent's own artefact at the version recorded in `evidence.upstream_version`. Enforced mechanically, **over `REAL_ADAPTERS` only**, by the clauses in §5, and bound to a real CI run by §5.3. | measured: `metadata_invariants` asserts only non-emptiness today (`crates/agent-profile/tests/adapter_contract.rs:264-266`), which any placeholder satisfies; a URL-shape check alone is satisfied by `https://example.com/`. The `REAL_ADAPTERS` scope is load-bearing: `registry()` includes `Fake` in every debug build (`adapter/mod.rs:112-120`) and tests build in debug, so an unscoped Gate A would demand `docs/evidence/fake-0.0.0.md` (`adapter/fake.rs:24`) for a fixture with no upstream product — satisfiable only by hand-writing the exact artefact §7.4 exists to detect. The pinned-support assertion already uses `REAL_ADAPTERS` (`adapter_contract.rs:275`). |
| D5 | **Gate B.** Every `basis` begins with `measured: `, `cited: ` or `unmeasured: `, contains no newline, and **`unmeasured:` and `CapabilityState::Unknown` imply each other**. The prefix names the provenance of the evidence that **establishes the state**; any supporting fact of weaker provenance is marked inline. Separately, and manually (§5.1), a `basis` for any state other than `NotSupported` either names a specific bypass or states that a search found none. | measured: this is what the shipped adapters already do informally — `claude.rs:25-27` enumerates eight override variables, `codex.rs:29-31` four. The biconditional is what makes D7 enforceable: without it, an implementer reaches `Proven` by writing `NotGuaranteed` with an `unmeasured:` basis, and every mechanical gate still passes. reasoned: the prefix names the *decisive* evidence rather than the weakest fact mentioned, because a weakest-wins rule rewards deleting an inconvenient sentence — it would demote a 90%-measured claim for admitting one inference, so the cheapest compliance is a shorter basis. The no-newline rule protects the "one `Vec` entry is one line" contract (`output.rs:30`, `cli.rs:596-597`). |
| D6 | **Gate C.** `SupportLevel::Proven` requires non-empty `notes` and **at most one** `Unknown` capability claim. The pinned support list at `adapter_contract.rs:275-286` is **kept** alongside the new invariant. | owner ruling, 2026-09-16 (§4.1): V3:132-133 permits exactly this. measured: the pinned triple and the invariant catch opposite mistakes. Gate C never asserts that any adapter *is* `Proven`, so replacing the list would let a one-word demotion of `claude` pass the suite silently while adding a permanent stderr banner to the flagship adapter. |
| D7 | A capability that cannot be measured because measuring it needs an authenticated session is `CapabilityState::Unknown` with an `unmeasured:` basis. No enum variant is added. | reasoned: `NotGuaranteed` and `Conditional` are claims about a mechanism that was understood — both existing uses of `NotGuaranteed` describe a known leak path (`claude.rs:44-46`, `aider.rs:38-43`), and `Conditional` promises the conditions are enumerable (`codex.rs:43-45` lists three variables by name). Using either for "we could not log in" borrows the authority of a measurement that never happened. `NotSupported` is a positive negative claim and would defame a product that may isolate correctly. |
| D8 | `sandbox/probes/common.sh` gains `probe_behaviour <id> <default-location>`: snapshot `<default-location>` after install and before launch, launch the agent under `agent-profile` (not `--dry-run`), then record the delta at both the profile directory and `<default-location>`. §7.3 fixes the whole step order explicitly. | measured: today `probe_agent_profile` records `cargo build`, `--version` and a dry run (`sandbox/probes/common.sh:22-26`), so it captures what `agent-profile` *plans*, never what the agent *does*. Ordering is load-bearing: `claude.sh:5-7` runs `claude --version` and `claude --help` natively before `probe_agent_profile`. The baseline is equally load-bearing and ordering alone does not supply it: the install runs first and executes vendor code (§9), so an installer that creates the default location would otherwise be attributed to the wrapped launch, making `ConfigIsolation` read worse than the truth. |
| D9 | Probes run in the Actions Sandbox workflow as a **matrix, one job per agent**, bounded by a concurrency group and a deduplicated, length-capped agent list. | measured: only the Podman branch of `sandbox/run.sh` creates a private ephemeral image store (`:84`) and deletes it at cleanup (`:105-108`); the Docker branch does `chmod a+rwx "$out"` and nothing else (`:88`). Podman is absent from the owner's machine; the workflow sets `AGENT_PROFILE_SANDBOX_ENGINE: podman` (`.github/workflows/sandbox.yml:43`) on a runner VM that is discarded. A matrix is required rather than a loop: each invocation builds a fresh image (`sandbox/run.sh:124`) containing a Rust toolchain, so twelve sequential probes would not fit the job's 60-minute cap (`sandbox.yml:35`), and one hanging agent would consume the budget of all the others. The bounds are required because the matrix removes that cap: `timeout-minutes` is a job attribute, and `sandbox.yml` has no `concurrency` key today while `docs.yml:17-19` shows the repository's idiom. |
| D10 | `ProfilePresence::Known` is **retained**, unconstructed, and documented as reserved by V3 §8 for native-profile adapters — none of which SP4 ships. The `status` label stays. | owner ruling, 2026-09-16. cited: V3:346-348 defines it normatively, V3:350 states "A physical directory is not universally required", and V3:373-374 reserves it for native-profile adapters. measured: the variant has exactly two occurrences — `metadata.rs:102` and `cli.rs:739` — and the default `presence()` returns only `Materialized` or `Absent` (`adapter/mod.rs:48-58`). The original anomaly called it a dead variant; the oracle's answer is that it is not dead, it is unclaimed. A doc comment at `metadata.rs:102` records that, so the next reader does not re-file it. |
| D11 | An SP4 adapter ships with `conflicts: &[]` unless a probe transcript shows the option in that agent's `--help` at the pinned version — **except that an adapter whose own mechanism is a flag always declares that flag**, with no probe required. | reasoned: V3:836 says "If the wrapper cannot prove a conflict, it must not guess", which gives the default; V3:833-834 says the wrapper "must fail before launch" where an option "controls the same profile mechanism", which gives the exception. For a config-file adapter the proof is by construction: `config_file_arg_plan` emits its own flag and then appends the user's arguments after it (`adapter/mod.rs:197-198`), so a user-supplied second spelling of the same flag reaches the agent after ours. Aider, the shipped config-file adapter, carries exactly such a list (`aider.rs:56-59`). Without the exception, §9 outcome 2 would ship Continue or Amp with an empty list and `agent-profile continue work -- --config other.yaml` would silently defeat the profile it selected. **The exception covers the short form too**, where the agent documents one: `ConflictOption` carries `short: Option<char>` (`metadata.rs:70-74`) and Aider declares `short: Some('c')` (`aider.rs:58`), and `-c other.yaml` defeats the profile exactly as the long spelling does. Restricting the exception to the long spelling would leave that hole permanently open for an outcome-2 adapter, which by definition never gets a transcript. Only *abbreviated* long spellings — Aider's `--conf`, `--confi` — still require one, because which prefixes an agent accepts cannot be derived. |
| D12 | All nine ship in one wave rather than split by measurability. | cited: research on 2026-09-16 established that every one of the nine installs and runs `--help`/`--version` with no account, and every one needs an account to do real work. The proposed split axis puts all nine in the same bucket. |
| D13 | The probe harness is repaired **before** the probe run: `probe_record` propagates failure, every recorded command runs under a timeout, `cargo build` runs once per job rather than once per recorded step and its failure is recorded, and `probe_behaviour` takes a baseline snapshot. | measured, and this is the finding that would have wasted the whole run: `probe_record` ends with `set -e` (`sandbox/probes/common.sh:18`), and a shell function returns the status of its last command, so `probe_record` **always returns 0**. Every probe script ends in `probe_agent_profile`, which ends in `probe_record dry-run`, so a probe whose install failed and whose every later command returned 127 still exits 0 and the Actions job is green. There is no `timeout` anywhere in `common.sh` or `run.sh`, and `cargo build` at `common.sh:23` is unguarded, so under `set -eu` (`:7`) a build failure aborts the probe after the install, leaving transcripts that look complete with no behavioural step. Scope note: under D9's matrix the crate is built once per *job*, which is once per agent; building it once across all agents is not achievable and is not claimed. |

### 4.1 D6: the owner's ruling, and a correction to how it was framed

**This was never an oracle conflict, and the panel round that called it one was wrong.** V3:132-133
grants a *permission* — an adapter **may** be `Proven` with one `Unknown` — and declining a permission is
not divergence. Nothing in V3 requires any particular adapter to *be* `Proven`; §37 asks only that all
twelve have evidence. The stricter gate was therefore always available without anything to report, and
§1.1 carries no row for it. The ruling below stands on its merits, not on deference to the oracle, and
the record is corrected here so that a later reader does not mistake a product decision for a forced one.

The draft rule was "`Proven` requires that no capability claim is `Unknown`". V3 states the permission
directly:

> `agent-profile-implementation-spec-v3.md:132-133` — "An adapter may be `Proven` while one capability
> remains `Conditional`, `NotGuaranteed`, or `Unknown`."

Under V3's rule an adapter whose config and state isolation are measured and whose credential isolation
alone is `Unknown` may be `Proven` — the exact shape §6 predicts for most of the nine. Under the draft
rule all nine would have been `Experimental`, each carrying D3's stderr hedge on every launch forever.

**Owner ruling, 2026-09-16: follow V3.** `Proven` tolerates at most one `Unknown` capability. The support
level keeps meaning "the mechanism is proven"; the capability matrix carries the nuance, and D2 makes that
matrix unconditional, which is what stops `proven` from overclaiming. A second `Unknown` capability means
the mechanism itself is not understood, and that is what `Experimental` is for.

## 5. The evidence bar, as contract-suite invariants

All gates are assertions in `metadata_invariants`
(`crates/agent-profile/tests/adapter_contract.rs:241`). **Gates A and C iterate `REAL_ADAPTERS`, not
`registry()`** — see D4 — while Gate B applies to every adapter including `fake`, which has real claims
and must not model bad ones.

```text
Gate A   source_url is a URL, or the literal "measured" with non-empty notes        [SP4a]
         AND docs/evidence/<id>-<upstream_version>.md exists                        [fold commit, §5.2]
         AND it contains the adapter's mechanism token, unless the adapter is
             registered under §9 outcome 2 (its transcript records a failed probe
             and cannot contain an observation that never happened)                 [fold commit]
         AND every file in docs/evidence/ matching <id>-<version>.md is referenced
             by exactly one adapter, and no file there fails to match that pattern
             except README.md                                                       [fold commit]
Gate B   every basis starts with "measured: ", "cited: " or "unmeasured: "
         AND contains no newline
         AND starts with "unmeasured: " if and only if its state is Unknown
Gate C   support == Proven  =>  notes is non-empty
                            AND at most one capability claim is Unknown
         AND the pinned support list still holds for claude, codex and aider
```

`upstream_version` is constrained to `[A-Za-z0-9._-]+` so that `<id>-<version>.md` always names one
unambiguous file: it is an unconstrained `&'static str` today (`metadata.rs:20`) and Gate A builds a path
from it. Throughout this document the filename uses the adapter's **id**, never its executable name, which
differs for Kiro, Cursor and Continue (§8).

Gate A's last clause is the converse of the others. Without it the evidence directory and the registry
drift apart silently — an orphaned transcript asserts a measurement about a version nobody supports and
looks, at a glance, exactly like current evidence. The pattern requirement is what keeps the clause strong:
scoping it only to matching files would let a stray `notes.md` sit there unexamined.

### 5.1 The manual surface

These rules cannot be asserted, and are named here together so no one mistakes them for gates:

1. **Gate B's content half** — whether a `basis` sentence describes a real bypass, and whether it omits an
   inconvenient fact. No test can judge a sentence; D8's transcript is what a reviewer checks it against.
2. **The standing red flag** — no shipped adapter claims `CredentialIsolation: Supported`: Claude is
   `Conditional` (`claude.rs:38`), Codex `Conditional` (`codex.rs:42`), Aider `NotSupported`
   (`aider.rs:46`). Every one of the nine has a documented API-key or token variable that bypasses its
   mechanism (§8). An SP4 adapter arriving as `Supported` means the measurement is absent or wrong.
3. **Transcript content** — §5.3 binds a transcript to a real run and proves the committed bytes are the
   uploaded bytes; it cannot judge whether the excerpt inside was the right excerpt.
4. **§8's correction duty** — "where a probe contradicts a row, the probe wins" requires someone to re-read
   §8 after the run. Nothing enforces it.
5. **Mutant proof of new tests** (§10) — `just mutants` exists (`justfile:68-69`) but no CI job or hook
   runs it.
6. **Gate C's `notes` clause** is a non-emptiness check; `notes: "-"` satisfies it. §5.3, not Gate C, is
   what makes evidence expensive to fake.
7. **Every `custody: off-ci` transcript** — §5.3's job skips them by design, so the one escape from all of
   this machinery is reviewed only by a person. Gate A requires `custody: ci` for any transcript backing a
   `ConfigIsolation` or `StateIsolation` claim, which is what keeps the escape narrow; no SP4-era
   transcript should be `off-ci` at all, because §2.2 puts its only legitimate use out of scope.

### 5.2 Why Gate A is split across two commits

`docs/evidence/` does not exist in the tree today (`docs/` holds `superpowers/`, two ledgers and
`dev-tooling.md`), and the three shipped adapters name versions — `claude` `2.1.270` (`claude.rs:23`),
`codex` `0.153.4` (`codex.rs:27`), `aider` `0.86.2` (`aider.rs:32`) — whose transcripts were never
committed. A Gate A that asserted transcript existence inside SP4a would turn the branch red for three
adapters, and the only artefact that could make it green is produced by the probe run that D1 sequences
*after* SP4a.

So Gate A lands in two pieces: the shape half in SP4a, the transcript half in the fold commit that also
adds the transcripts. Neither commit is ever red. This is the one place where an SP4a deliverable is not
buildable against code that exists now, and splitting it is what preserves D1's premise rather than
quietly breaking it.

### 5.3 Binding a transcript to the run that produced it

Every gate above checks a file's *shape*. None of them checks that the file came from anywhere, and a
convincing transcript — real run URL copied from the public Actions tab, plausible `--help` excerpt, a
column of zero exit codes — can be written by hand in minutes and passes all of them. Renaming
`<id>-2.1.0.md` to `<id>-3.0.0.md` and editing one version string passes them too, and reads in a diff as
a textbook evidence refresh.

**The machine assembles; the human reviews; the job compares.** The probe job writes the *finished*
`<id>-<version>.md` — custody block included — into `sandbox-transcript-<id>`. A maintainer downloads it,
reads it, and commits it **byte for byte or not at all**. No human edits a transcript; an excerpt that
needs changing means the probe script needs changing and the probe re-runs.

That ordering is what makes verification simple enough to trust. The earlier draft had a human assemble
the committed file from uploaded ingredients, which meant the committed bytes were by construction *not*
the uploaded bytes, so no hash could ever match — and it forced the job to parse attacker-supplied text to
discover what to compare. Both problems disappear when the artifact and the commit are the same bytes.

**The verification job**, on `pull_request` in `ci.yml`, does this for each adapter in `REAL_ADAPTERS`
whose transcript changed in the pull request:

1. Take the agent id and `upstream_version` **from the registry**, never by parsing the filename. The
   filename-to-id map is not invertible: `upstream_version` permits `-` (§5), so `cursor-1.0.0-beta.1.md`
   splits two ways.
2. Read `run-id:` from the committed file, require it to match `^[0-9]+$`, and pass it to the API through
   the environment — never through `${{ }}` expression interpolation.
3. Assert the run belongs to this repository, is the Sandbox workflow, concluded `success`, **has a head
   SHA equal to the `harness-commit:` the transcript records, and that commit is an ancestor of the base
   branch.**
4. Assert `sandbox-transcript-<id>` contains a file byte-identical to the committed one.

**Step 3 is not optional and the draft omitted it.** Without a commit binding, a person with push access
dispatches Sandbox on a throwaway branch carrying an edited `sandbox/probes/<id>.sh` that prints a
fabricated `--help`; the run is genuinely green, the artifact is genuine, the bytes match exactly, and the
forging branch never appears in the pull request. The harness commit was already recorded in §7.4 and
nothing checked it.

**It must be `pull_request`, never `pull_request_target`.** The reflex when a fork's token cannot reach
the API is to switch, which would run a base-repository token against pull-request-supplied bytes. If fork
coverage is wanted, it is a separate problem and not solved that way. The job needs `actions: read` and
nothing else, and **it is a separate job from the probe**, which keeps `contents: read` — the probe
executes third-party installers and must remain the least privileged thing in the repository (§9). It
lives in `ci.yml` because `sandbox.yml` filters on `paths: sandbox/**` (`.github/workflows/sandbox.yml:12-15`)
and would never fire on an evidence-only pull request; `ci.yml` has no paths filter (`.github/workflows/ci.yml:3-8`).
It is a required status check, or it is advice.

**Expiry fails closed.** Artifacts expire, and the earlier draft degraded to run-exists,
correct-workflow, concluded-success and an ancestry check — none of which is a function of the agent, the
version, or the bytes, so the original run of `<id>-2.1.0.md` satisfies all of them for a forged
`<id>-3.0.0.md`. That degradation restored both forgeries this section exists to stop, and a green
`pull_request` run in `test` mode — which probes nothing — satisfied it too. So: a transcript is committed
while its artifact still exists, the workflow sets an explicit artifact retention long enough to make that
routine, and **a transcript whose artifact has expired cannot be introduced or modified**. An existing
verified transcript stays verified; re-verification after expiry is not attempted, because a check that
cannot fail is not a check.

A transcript marked `custody: off-ci` (§7.4.1) is exempt from this job and reviewed by hand. Gate A
therefore **requires `custody: ci` for any transcript backing a `ConfigIsolation` or `StateIsolation`
claim**: without that requirement, `off-ci` is a one-word opt-out from every mechanical control here, and
§2.2 puts the only legitimate use of `off-ci` — authenticated measurement — out of scope for SP4, so no
SP4 transcript should carry it at all.

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

### 6.1 Tier-B adapters and V3:139-140

V3 singles out Aider, Amp, Continue and Cursor: they "must not be presented as guaranteed isolated
accounts unless evidence proves that claim". Three are SP4 adapters, and under D6 each could reach
`support: proven` on a config-isolation measurement while its credential claim stays `Unknown`.

**Owner ruling, 2026-09-16: the matrix satisfies the requirement**, on these grounds. `proven` is a
statement about the mechanism, not about the account. D2 renders the capability states unconditionally
and in the same block, so a user cannot read `support: proven` without also reading
`credentials: unknown` directly beneath it — the report has no mode that shows one and hides the other.
"Presented as a guaranteed isolated account" would require a surface that asserts isolation without that
qualification, and SP4 ships none.

**Three consequences are binding, and they are the price of the ruling.** D2's matrix may not become
conditional or opt-in in a later sub-project without revisiting it. SP5's JSON output must carry the
capability states in the same object as the support level. And **every other surface that names a support
level must name the capabilities too** — which the draft broke in its own §11, by proposing a README
column listing tier-B agents as `proven` with no caveat anywhere near them. That is fixed in §11; the
general rule is recorded here because the next surface to show a support level will face it again. All
three are in `TODO.md`.

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

When `metadata().support` is not `Proven`, the launch is real rather than a dry run, **and no report is
being rendered**, the CLI writes one line to stderr before the launcher runs:

```text
agent-profile: kiro is experimental: config not guaranteed, credentials unknown. Run with --dry-run for detail.
```

Every capability whose state is **not `Supported`** is listed, in `Capability::ALL` order, using the
report's own names — `config`, `credentials`, `state` (§7.1) — so one vocabulary covers both surfaces. If
every capability is `Supported`, the line names the support level alone. The `--verbose` exclusion matters
because that report also goes to stderr on a real launch (`cli.rs:603-608`), and pointing a user who just
read the full matrix at `--dry-run` would be absurd.

**No suppression key ships in SP4, and that is a consequence of D6's ruling.** The draft carried a
`hide_support_warning` configuration key, justified by the argument that a warning which can never be
cleared is trained away within a week. That argument described the *stricter* Gate C, under which the
hedge would have fired on nine of twelve adapters forever. Under the ruling it fires only for an adapter
with two `Unknown` capabilities — one whose mechanism was never established at all.

How often that happens is not known, and §6 says why: it depends on which agents write state files before
they need the network, which is exactly what the probe run measures. The key is also not free — the
schema is strict (`crates/agent-profile/src/config.rs:219` rejects any unrecognised key as
`"unknown key"`), so shipping it means a configuration file written for the new binary is rejected by
every earlier one. Breaking configuration compatibility to mute a warning whose frequency nobody has
measured is the behaviour this document forbids everywhere else, so the key waits for the evidence. If
the probe run shows the hedge is common, SP5 adds suppression with a measured reason.

The hedge's remaining limits are recorded in §12 rather than papered over.

### 7.3 Probe harness (D8, D13)

`common.sh` is repaired first:

- `probe_record` returns the recorded status and accumulates failures, so a probe that failed exits
  non-zero. This is D13's core fix; without it the probe run's green light carries no information.
- every recorded command runs under `timeout <n>`, and a timeout status is recorded as the "cannot run
  non-interactively" fact rather than being left to the job cap.
- `cargo build` runs once per job, not once per recorded step, and its failure is recorded rather than
  aborting the script silently.

**Every probe script then follows this order exactly.** The order is part of the contract, because
`probe_behaviour`'s delta is only attributable if nothing has run the agent before it:

1. `install` — the only step that runs vendor code before a snapshot exists.
2. `version` — the agent's own `--version`. This is the value `evidence.upstream_version` takes, and it is
   recorded here, before anything has launched the agent for real.
3. `help` — the agent's own `--help`, the artefact Gate A reads the mechanism token from.
4. `strings` — the bypass-variable excerpt.
5. `baseline` — record **both** `<default-location>` and the profile directory, as the install and
   `agent-profile`'s own initialization left them.
6. `behaviour` — `probe_behaviour <id> <default-location>`: launch under `agent-profile`, then record the
   delta at both locations against step 5.

**Both baselines are load-bearing, and the earlier draft had only one.** `agent-profile` creates the
profile directory itself before the agent starts — `adapter.initialize(&planned)?` at
`crates/agent-profile/src/cli.rs:602` — and for a configuration-file adapter it also writes the minimal
accepted file of §8.4. Those bytes are ours, not the agent's. Baselining only the default location
attributes them to the agent and errs *optimistically*: an agent that ignored `--config` entirely would
still leave a non-empty profile-directory delta, which §6's table would read as proof that its config
landed under the profile.

`--version` and `--help` run at steps 2 and 3 rather than after the launch. They are unwrapped
invocations, but they are read-only observations of a freshly installed agent, and running them after a
real launch would mean reading the recorded version and the mechanism token out of an installation that
had already executed vendor code once with network access. The baseline at step 5 is taken after them for
exactly that reason: it must capture everything that existed before the *wrapped* launch, including
anything steps 2 and 3 caused. Every existing script runs version and help before the wrapped run
(`sandbox/probes/claude.sh:5-7`); what changes is the baseline between them, not their position.
`agent-profile --version` inside `probe_agent_profile` (`common.sh:24`) is not an invocation of the agent
and does not constrain this order.

The default location is a per-agent fact, so each probe script passes it; `common.sh` cannot derive it.
The baseline is recorded in the transcript, not merely used to compute the delta: the install ran vendor
code before anything else, and a reviewer needs to see what it had already created.

A probe never supplies a credential and never waits for input.

### 7.4 Evidence transcripts (D8)

`docs/evidence/<id>-<version>.md` is **written whole by the probe job** (§5.3) and holds, for one probe
run, in this order:

```text
custody: ci
run-id: <digits>
run-url: <url>
harness-commit: <40 hex>
---                          <- everything below this line is the hashed body
install: <command, with the resolved version>
exit-codes: <one line per recorded step, verbatim>
version: ...
help: <excerpt containing the mechanism token>
strings: <candidate bypass variables>
baseline: <default location, and the profile directory>
delta: <both locations, against the baseline>
```

**The hashed body starts after the `---` line and runs to the end of file, including the trailing
newline.** A file cannot contain its own hash, so the header above the marker is excluded and everything
else is inside — in particular `exit-codes:`, which the earlier draft placed in the custody header and
therefore left unbound. Those codes decide which of §9's four outcomes a probe landed in; leaving the one
field that determines the verdict outside the signed region would have made §5.1's claim about §5.3 false
of the bytes that matter most. The four header lines are bound instead by the API check: `run-id` and
`harness-commit` are compared against the run itself.

§5.3 is what makes these fields load-bearing rather than decorative.

Captured output is untrusted vendor text. Before it becomes a committed file it is stripped of terminal
control sequences and bounded in size; an excerpt that must be truncated says so at the truncation point.

**Retention.** One transcript per adapter is live at a time: the one Gate A resolves from the current
`upstream_version`. A re-measurement replaces the file it supersedes — git history holds the old one, and
Gate A's last clause fails if an orphan is left behind. An `unknown` transcript from a failed probe
(§9 outcome 2) is deleted when the agent is later probed successfully; leaving it would be a durable false
statement that the agent cannot be installed.

**Probe scripts take an optional version argument.** The default resolves to whatever the registry serves,
which is how a version is discovered; passing the recorded version is how a measurement is repeated.
Today's probes pin nothing (`sandbox/probes/claude.sh:4`, `aider.sh:5`), and CONTRIBUTING's pinning
guidance covers the *interpreter*, not the package (`CONTRIBUTING.md:108-109`).

#### 7.4.1 Off-CI custody, for authenticated measurement

A transcript produced outside CI has no workflow run URL, so it carries a different, explicitly weaker
custody block: the sandbox and its version, the host platform, the date, and who performed it. Its first
line reads `custody: off-ci` so §5.3's verification job skips it and a reviewer never mistakes it for a
CI transcript.

This shape exists because §6's upgrade path would otherwise be unreachable: an authenticated measurement
cannot run in CI (§2.2), so the one transcript that could ever justify raising `CredentialIsolation` is
the one that cannot carry CI custody. The measurement still happens in a disposable sandbox — SP2 used
Sandboxie-Plus for exactly this (SP2 design D9) — never on the host. Performing such a measurement is out
of scope for SP4; defining how it must be recorded is not.

### 7.5 Workflow (D9)

`sandbox.yml` ends with **three** jobs, and naming only two is how the draft silently deleted the one CI
that keeps the harness working. The `test` job runs `sandbox/run.sh test` for the `pull_request` trigger
and for a `workflow_dispatch` with `mode == test`; it is the job the workflow's own header promises
("A pull request that changes the harness runs it in `test` mode only … so the harness itself stays
working", `.github/workflows/sandbox.yml:8-9`). It is unchanged by SP4 except that the existing single
job is split, and it must not be dropped: the input rename below would otherwise leave the surviving job
running `sandbox/run.sh probe ""`, which `sandbox/run.sh:51-54` refuses with exit 2.

The `agent` input is renamed to `agents` and accepts a whitespace- or comma-separated list, or `all`. A
`prepare` job — **which runs only for `workflow_dispatch` with `mode == probe`** — parses it, validates
each name against `[a-z0-9-]` and against the existence of `sandbox/probes/<name>.sh`, **deduplicates**,
caps the list at **24** entries, and emits a JSON array. An empty or unknown name fails the job loudly; a
naive split-and-loop would iterate zero times and report success having probed nothing, and a duplicated
name would double the runner cost while colliding on the artifact name the matrix keeps unique.

The event gate is not optional: on `pull_request` there are no `workflow_dispatch` inputs
(`.github/workflows/sandbox.yml:12-15`), so `inputs.agents` is empty and an ungated `prepare` would fail
every pull request that touches `sandbox/**` — reddening the very CI that exists to keep the harness
working. The cap is 24 rather than 12 because §9's probe run needs all twelve at once and a cap at the
current registry size would make the thirteenth adapter a workflow edit.

A `probe` job then runs `strategy: { fail-fast: false, matrix: { agent: <that array> } }`, so one agent's
failure or hang cannot consume another's budget, and each job keeps its own 60-minute cap. A
`concurrency` group prevents a re-dispatch from running twelve more installer jobs alongside twelve
already in flight; `docs.yml:17-19` is the repository's idiom.

Each probe job uploads two artifacts. `sandbox-transcript-<id>` holds the finished transcript §5.3 hashes
against, and **a bounded summary of `diff.txt` goes inside it**, not alongside: `diff.txt` is the whole
container delta (`sandbox/run.sh:154`) and therefore the only record that can show a write to a location
nobody predicted, which is the only way §9 outcome 4 — "the agent exposes a different mechanism from the
claimed one" — is ever detected. The transcript's own delta is scoped to two locations chosen in advance,
so routing the unscoped observation into the unhashed, expiring artifact would leave the broadest evidence
the least protected. `sandbox-logs-<id>` keeps the raw `build.log`, `output.log` and the full `diff.txt`;
the split exists to keep megabytes of toolchain-download noise out of the file a human reviews (§5.1), not
to discard the container delta.

Both uploads set an explicit retention period, long enough that committing a transcript inside its
artifact's lifetime is routine rather than a race — §5.3 depends on that window, and the current upload
step sets no retention at all (`.github/workflows/sandbox.yml:53-59`).

No secret is added to any job.

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
the id for Kiro, Cursor and Continue; probe scripts and transcripts are named by **id**. The install
column is cited, not measured, and the probe run tests it: an install command is itself an unverified
claim until a probe executes it.

Continue and Amp each declare their own mechanism flag in `conflicts` without waiting for a probe, per
D11's exception.

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

The divergence from V3 §2 is recorded in §1.1. If the probe finds no OpenCode-specific data-directory
variable, the adapter sets `OPENCODE_CONFIG_DIR` alone and declares `CredentialIsolation` and
`StateIsolation` `NotSupported`, each with a basis naming the directory that does not move. Note the
consequence under D6: two non-`Unknown` negative claims do not block `Proven`, so OpenCode can be `Proven`
*and* honestly state that it isolates configuration only.

### 8.3 Cline's mechanism is undecided by design

`--config` selects a settings directory and `--data-dir` an isolated state directory, but `--data-dir` is
documented as also enabling sandbox mode — an isolation flag that changes agent behaviour beyond
isolation. `CLINE_DATA_DIR` is a documented environment equivalent that may not carry that side effect.
The probe answers three questions in order: does `CLINE_DATA_DIR` isolate state; does it enable sandbox
mode; does `--config` work alongside it. The answer selects one of three shapes:

1. `CLINE_DATA_DIR` alone — the existing `env_dir_plan`, no new helper. This is a divergence from V3 §2,
   recorded in §1.1: it uses neither mechanism V3's table names.
2. `CLINE_DATA_DIR` plus `--config` — a new helper combining one variable and one flag.
3. `--config` plus `--data-dir` — a new helper emitting two flag/value pairs, and the only shape V3's
   table describes.

`config_file_arg_plan` cannot express 2 or 3: it emits exactly one flag/value pair
(`adapter/mod.rs:197`) drawn from the first `File`-kind path (`:191-195`). Whether SP4b adds a helper, and
which, is settled by measurement. No helper is written speculatively. Shapes 2 and 3 pass a flag, so under
D11 the adapter declares that flag in `conflicts`.

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

**The commit is manual and human-performed, and the human does not edit.** A maintainer downloads the
artifacts, reads each transcript, and commits them **byte for byte** together with Gate A's transcript
half (§5.2); §5.3's job then verifies each committed file against its run. A transcript that looks wrong
is not corrected by hand — the probe script is corrected and the probe re-runs, because the moment a human
edits the bytes, §5.3 can no longer bind them to anything. The probe job keeps `permissions: contents: read`
(`.github/workflows/sandbox.yml:28-29`) and **must never be granted write access**: it runs
`npm install --global` and vendor install scripts for twelve third-party packages, any of which can
execute arbitrary code in a postinstall hook. A repository-write credential in that job would let one
compromised package push to the default branch.

Four outcomes, and every probe lands in exactly one:

1. **Mechanism confirmed** — the token appears at the resolved version and the behavioural delta shows the
   files moved. The adapter ships with measured bases.
2. **Probe failed** — the agent will not install, or will not run non-interactively. The adapter is still
   written, from cited evidence, with `ConfigIsolation: Unknown`. That is a second `Unknown` alongside
   credentials, so D6 makes it `Experimental`, which is correct: nothing about its mechanism was
   established. Its evidence entry records `upstream_version: "unknown"` and a
   `docs/evidence/<id>-unknown.md` transcript recording the failure, which Gate A exempts from the token
   clause — a probe that never ran the agent cannot have observed the token, and requiring the string
   anyway would reduce Gate A to a substring test that any sentence satisfies. §7.4's retention rule
   removes that file when the agent is later probed successfully. The divergence from V3:110 is recorded
   in §1.1.
3. **Mechanism disproved** — the agent installs and runs, and the claimed token is absent from its
   artefacts. Gate A then forbids registration. **The adapter is not registered**, its row moves to
   `TODO.md` as tracked debt with the transcript as evidence, and — because this reduces V0.1 below V3's
   twelve adapters (§1.1) — **the owner signs off before SP4b proceeds**. This is an explicit exception to
   "no adapter is dropped", which covers outcome 2 only, and it is the most valuable result a probe can
   produce.
4. **Mechanism differs** — the agent exposes a *different* mechanism from the claimed one. §8's table is
   corrected and the adapter is written against what was measured.

No probe failure is worked around by installing the agent on the owner's host. A disposable local sandbox
remains available for a later authenticated measurement under §7.4.1.

## 10. Testing (V3 §34)

| Suite | Addition |
|---|---|
| `tests/adapter_contract.rs` | gates A-shape, B and C in `metadata_invariants` (SP4a); Gate A's transcript clauses in the fold commit (§5.2); one `expected()` row per new adapter (the `panic!` at `:131` makes a missing row a failure, not a silent pass); one `conflict_contract` row each, non-empty for Continue and Amp per D11. |
| `tests/adapters_e2e.rs` | one row per new adapter, launching `fake-agent` under the adapter's own executable name, proving the plan reaches the child. No real agent runs in CI. |
| `src/output.rs` unit tests | the `support:` and `isolation:` rendering for each `CapabilityState` spelling; the `basis` line in both modes; **a capability with no claim renders `not declared` and does not panic** (§7.1). **Re-baseline the positional assertions at `output.rs:392-393` and `:400`.** |
| `tests/launch.rs` | the D3 stderr hedge: present for an `Experimental` adapter, absent for a `Proven` one, absent on a dry run, **absent under `--verbose`**, listing every capability that is not `Supported` (including a `NotGuaranteed` one, which the earlier draft would have omitted), and on stderr rather than stdout. **Re-baseline `launch.rs:143`.** |
| Probe harness | `probe_record` returns non-zero for a failed command — the D13 regression that would otherwise make the whole probe run meaningless. Testable without any agent: record `sh -c 'exit 3'`. |
| `ProfilePresence::Known` | the variant stays; a test asserts the `status` label renders it, so the arm at `cli.rs:739` is not dead code that a later reader deletes. |

Every new test is proven non-vacuous with a logic mutant, per the repository's assertion-strength
discipline: a test that cannot fail is a test that certifies nothing.

## 11. Documentation

- `README.md`: the supported-agent table grows to twelve. It carries a support-level column **and a
  capability summary in the same row**, so `proven` never appears without its credential caveat beside
  it — §6.1's argument is only true if it holds on every surface, and a support column alone would be
  exactly the presentation V3:139-140 forbids. The table also states the Windows shim consequence of §12.
- `CONTRIBUTING.md`: how to add an adapter, pointing at the gates and at `docs/evidence/`.
- `docs/evidence/README.md`: what a transcript is for, both custody shapes (§7.4, §7.4.1), how §5.3
  verifies one, how to refresh one, and that a refresh replaces rather than accumulates.
- `ROADMAP.md`: SP4 done, SP5 next; the `ProfilePresence::Known` note added at `8741ea0` is resolved by
  D10 — the variant is reserved, not dead.
- `TODO.md`: any capability left `Unknown` that an authenticated measurement could settle; any adapter
  dropped under outcome 3 of §9; any adapter registered with `upstream_version: "unknown"`; and §6.1's two
  binding consequences for SP5.

## 12. Known limits

- **Credential isolation is unproven for all nine.** SP4 states this rather than fixing it; fixing it
  needs a paid account per vendor. Under D6 an adapter may still be `Proven` with that single `Unknown`,
  which is why §7.1's matrix is mandatory rather than optional: the support level alone would overclaim,
  and §6.1's ruling depends on it.
- **Evidence is a snapshot.** A transcript pins one version on one day. Drift detection is `doctor` (SP5),
  and until then a mechanism can change upstream without the repository noticing.
- **§5.3 proves provenance, not content.** It proves the committed bytes are the bytes a real green run of
  this workflow produced, from a harness commit on the base branch; it cannot prove the probe script
  captured the right excerpt. Its cost falls entirely on someone with push access — it is not a defence
  against an outside attacker, who cannot commit at all.
- **A transcript cannot be introduced after its artifact expires.** That is deliberate (§5.3 fails
  closed), but it means a probe run left uncommitted past the retention window must be re-run, and the
  window is a workflow setting someone can shorten without noticing what depends on it.
- **The behavioural probe proves where files landed, not that nothing leaked.** A delta shows the config
  directory moved; it cannot show that no credential was read from a shared location.
- **`StateIsolation` measurement is partial** for agents that write history only after a network call.
- **The hedge can be erased by the agent it warns about.** It is written to stderr immediately before
  `exec` hands the terminal over, and an agent that opens a full-screen alternate-screen TUI may clear it
  before it is read. The durable disclosures are the report and the README; a first-launch acknowledgement
  belongs with `create` in SP5.
- **The hedge cannot be turned off in SP4.** Suppression was deferred (§7.2) because its frequency is
  unmeasured and the strict schema makes a new key a compatibility break. A user who finds the line
  repetitive has no escape hatch until SP5.
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
| Q6 | Which agents expose an abbreviated conflicting spelling in `--help`? | probe transcripts; D11 gives the exact-flag answer without a probe |
| Q7 | Does any of the nine have a native named-profile concept? | probe run; if one does, it is the first constructor for `ProfilePresence::Known` (D10) |
| Q8 | What is the smallest file content Continue and Amp each accept that sets no option? | probe run (§8.4); no contents are guessed |
| Q9 | Does `Proven` tolerate one `Unknown` capability, per V3, or not? | **answered**: owner ruling 2026-09-16, follow V3 (§4.1) |
| Q10 | Is `ProfilePresence::Known` deleted or retained? | **answered**: owner ruling 2026-09-16, retained and documented as reserved (D10) |
| Q11 | Does `support: proven` with a visible matrix satisfy V3:139-140 for tier-B agents? | **answered**: owner ruling 2026-09-16, yes (§6.1) |

No question in this table is answered by reasoning in SP4b's plan. Each is answered by a transcript, an
owner ruling, or recorded as unanswered — and an unanswered question produces `Unknown`, never a guess.

## Stand-downs

- `DISCARDED-BELOW-FLOOR`: whether the GitHub runner ships Podman for `AGENT_PROFILE_SANDBOX_ENGINE: podman`
  (`.github/workflows/sandbox.yml:43`) — fails closed at `sandbox/run.sh:64`, which exits 2 with a clear
  message rather than proceeding unsafely.
- `DISCARDED-BELOW-FLOOR`: an embedded newline in a hand-written `basis` literal — now asserted by Gate B
  (§5), so the remaining risk is zero rather than merely low.
- `DISCARDED-BELOW-FLOOR`: long `basis` strings wrapping in a narrow terminal — display only; no contract
  claims a maximum width.
- `DISCARDED-BELOW-FLOOR`: `notes: "-"` satisfying Gate C's non-emptiness clause — real, and named in
  §5.1 item 6 rather than patched, because a length threshold is equally gameable and §5.3 is the control
  that actually raises the cost of faking evidence.
- `REJECTED`: a claim that a real adapter could reach `registry()` while omitted from `REAL_ADAPTERS`,
  making the two drift. `adapter/mod.rs:113-120` builds `registry()` from `REAL_ADAPTERS.to_vec()` and
  pushes only `Fake`, so that state is unconstructible.
