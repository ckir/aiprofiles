# Agent Profile Roadmap

v0.1 scope is fixed by the V3 specification. The build order follows spec §35, split into sub-projects.
Each sub-project has its own design spec and implementation plan under `docs/superpowers/`, and ends
green on Linux, macOS and Windows.

| SP | Deliverable | V3 §35 phases | State |
|---|---|---|---|
| SP0 | Scaffold — workspace, tooling, licence, community docs, CI, `fake-agent` fixture | — | **in progress** |
| SP1 | Core + explicit launch — profile-name validation, application root, TOML configuration with locked atomic writes, structured errors, `LaunchPlan`, Unix `exec` / Windows child launcher, passthrough, environment overrides, dry run, exit codes | 1, 3 | not started |
| SP2 | Architecture gate — adapter model, capability and evidence metadata, common contract suite, Claude Code, Codex CLI, Aider | 4A | not started |
| SP3 | Repository resolution — discovery, canonical identity, mapping storage, precedence, `resolve`, `current`, `status`, `link`, `unlink` | 2 + part of 5 | not started |
| SP4 | Remaining adapters — Gemini CLI, GitHub Copilot CLI, OpenCode, Cline CLI, Pi, Kiro CLI, Cursor Agent CLI, Continue CLI, Amp | 4B | not started |
| SP5 | Lifecycle and quality — `create`, `list`, `delete`, `repositories`, `doctor`, JSON output, shell completions, docs | rest of 5, 6 | not started |

`link`/`unlink` land with resolution (SP3), earlier than §35's Phase 5, so resolution never ships
without a way to create the mappings it resolves. All adapters still wait for the SP2 architecture
gate.

v0.1 is done when every item of spec §37 (Final Definition of Done) is checked.
