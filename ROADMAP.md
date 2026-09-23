# Agent Profile Roadmap

v0.1 scope is fixed by the V3 specification. The build order follows spec §35, split into sub-projects.
Each sub-project has its own design spec and implementation plan under `docs/superpowers/`, and ends
green on Linux, macOS and Windows.

| SP | Deliverable | V3 §35 phases | State |
|---|---|---|---|
| SP0 | Scaffold — workspace, tooling, licence, community docs, CI, `fake-agent` fixture | — | done (#1) |
| SP1 | Core + explicit launch — profile-name validation, application root, TOML configuration with locked atomic writes, structured errors, `LaunchPlan`, Unix `exec` / Windows child launcher, passthrough, environment overrides, dry run, exit codes | 1, 3 | done (#8) |
| SP2 | Architecture gate — adapter model, capability and evidence metadata, common contract suite, Claude Code, Codex CLI, Aider | 4A | done (#11) |
| SP3 | Repository resolution — discovery, canonical identity, mapping storage, precedence, `resolve`, `current`, `status`, `link`, `unlink` | 2 + part of 5 | done (#19) |
| SP4 | Remaining adapters — Gemini CLI, GitHub Copilot CLI, OpenCode, Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI, Amp | 4B | **next** |
| SP5 | Lifecycle and quality — `create`, `list`, `delete`, `repositories`, `doctor`, JSON output, shell completions, docs | rest of 5, 6 | not started |

`link`/`unlink` land with resolution (SP3), earlier than §35's Phase 5, so resolution never ships
without a way to create the mappings it resolves. The remaining adapters (SP4) build on the SP2
adapter model and contract suite.

`ProfilePresence::Known` was settled in SP4: it is **reserved, not dead**. Spec §8 defines a profile as
*known* when it can be identified from the adapter's documented profile mechanism rather than a
materialized directory. None of the twelve adapters is native-profile, so nothing constructs it — but
removing it would make the first such adapter a change to a public enum, and until then `presence()`
would have to report a profile the agent itself lists as `Absent`. The variant is documented at its
declaration and stays unconstructed.

## Tracked debt

**Antigravity CLI has no isolation mechanism, and the README says so.** Measured on 2026-09-22 against
`agy` 1.2.7: nothing relocates `~/.gemini/antigravity-cli/`, and credentials live in the OS keyring. It
is documented under "Agents that cannot be isolated" rather than shipped as an adapter, because the
`Adapter` trait materialises a profile directory and one that nothing reads would imply isolation the
user does not have. Re-measure if the vendor ships a configuration-directory variable or flag; at that
point it is an ordinary adapter candidate and a specification change, since V3's roster is closed at
twelve.

**Two contradictory action-pinning idioms.** `ci.yml` and `docs.yml` pin `actions/checkout` with the
floating tag `@v7`, while `release-plz.yml`, `release.yml` and `sandbox.yml` pin the same action by commit
SHA with a version comment. Whichever is right, the repository should not hold both — a reviewer cannot
tell which is the intended convention, and the newer Evidence job inherited the floating form by copying a
neighbour. Not urgent: every job that runs untrusted code already holds `contents: read` and no secret, and
the repository is public. Settle it before SP5 adds more workflows.

**`field()` in `sandbox/verify-transcripts.sh` reads the whole transcript, not the header its comment
claims.** The comment says "the value of a `label: value` line, from the header only"; the implementation
takes the first match anywhere in the file. What actually holds agent-authored body text out of a header
field is the two-space indent the assembler applies — measured: an agent `--version` line spelled
`version-extracted: 9.9.9` is excluded by that indent and by nothing else. The verifier's safety rests on
an undocumented coupling to `transcript.sh`. Scope `field()` to the header, or state the coupling where
both files can see it.

**A second `probe_candidates` sweep in one probe script would silently destroy the first.** The function
opens by resetting its counter and truncating `candidates.txt`, so a second call restarts step ids at
`candidate-1` and overwrites the first sweep's `.cmd`, `.exit-code` and `.txt`. Not reachable today —
exactly three scripts sweep (aider, amp, continue) and each sweeps once — but SP4b adds nine adapters. It
cannot be closed by a test: asserting accumulation asserts behaviour the code does not have, and pinning
today's behaviour codifies the defect. The function has to accumulate or refuse, which is a source change
that invalidates the capstone over that delta. Deferred by the owner at the SP4a test audit, 2026-09-21.

**The three shell suites leak a temporary directory per fixture.** Measured on the maintainer host:
16,537 `tmp*` entries in `TEMP` on 2026-09-21 and 22,136 on 2026-09-22, growing by roughly one per
fixture per run. `transcript.sh` and `verify-transcripts.sh` call `mktemp` and contain no `trap` and no
`rm -rf` at all; `probe-harness.sh` cleans some and still leaks. A `trap`-based cleanup in each is the
fix.

**Both `probe-tests` suites exceed a two-minute foreground budget, and only one of them is known for
it.** `sandbox/tests/verify-transcripts.sh` builds a git repository per fixture and commits into it, so
its cost scales with the fixture count exactly as `probe-harness.sh` does — yet only `probe-harness.sh`
is treated as the slow one, and a contributor who runs the other in the foreground loses the run to a
timeout. The mechanism is verified by reading; the wall-clock figure is not measured. A comment on the
`probe-tests` recipe naming both would close it.

v0.1 is done when every item of spec §37 (Final Definition of Done) is checked.
