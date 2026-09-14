# TODO

Near-term work. Sub-project scope lives in [ROADMAP.md](ROADMAP.md).

## SP2 open decisions

- [ ] **Launching `.cmd`/`.bat` shims without shell mediation** (spec §20, §23.2). npm installs agents on
      Windows as `<name>.cmd` shims, and Rust's `std::process::Command` runs batch files through `cmd.exe`.
      SP1 refuses `.bat` and `.cmd` executables outright (SP1 design D8); SP2 must decide how a real adapter
      launches a shim-installed agent directly.

## SP3 open decisions

- [ ] **Git repository discovery** (spec §13, §14). §14.2 requires Git's worktree metadata, so a plain
      upward walk for `.git` that ignores that metadata is not enough. Cover submodules, worktrees, nested
      repositories, symlinks and canonicalization failure.

## Housekeeping

- [ ] Enable GitHub private vulnerability reporting (see [SECURITY.md](SECURITY.md)). Branch
      protection is applied during SP0 (design §4.1).
- [ ] The MSRV (1.98) is not checked in CI; verify by hand with
      `cargo +1.98 check --workspace --all-targets`. Decide whether to add a CI job.
- [ ] `cargo install --path crates/agent-profile` also installs the `fake-agent` and `console-driver` test
      binaries. Not reachable from any gate or release (release.yml builds `--bin agent-profile`); revisit if
      source installs are ever documented.

- [ ] `console-driver` (test-only) drains the wrapper's stdout and stderr to EOF with no timeout; a descendant
      that inherits those pipes (for example a `FAKE_AGENT_SPAWN_SLEEPER` sleeper) keeps the driver alive past its
      60 s exit timeout. No current test combines a console event with a sleeper; bound the drains before adding one.

## Scaffold follow-ups

- [ ] Run `lefthook install` in each clone
