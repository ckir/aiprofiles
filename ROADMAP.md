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

**Two contradictory action-pinning idioms.** `ci.yml` and `docs.yml` pin `actions/checkout` with the
floating tag `@v7`, while `release-plz.yml`, `release.yml` and `sandbox.yml` pin the same action by commit
SHA with a version comment. Whichever is right, the repository should not hold both — a reviewer cannot
tell which is the intended convention, and the newer Evidence job inherited the floating form by copying a
neighbour. Not urgent: every job that runs untrusted code already holds `contents: read` and no secret, and
the repository is public. Settle it before SP5 adds more workflows.

v0.1 is done when every item of spec §37 (Final Definition of Done) is checked.
