# TODO

Near-term work. Sub-project scope lives in [ROADMAP.md](ROADMAP.md).

## SP2 known limits

- [x] **Launching `.cmd`/`.bat` shims without shell mediation** (spec §20, §23.2). Decided in SP2 (design D2):
      shims are refused, never parsed; the error names the `[agents.<id>] executable` setting.
- [ ] Aider conflict scan is shallow (spec §21): option values are not parsed, so a value equal to a conflicting
      spelling (`--message --config`) is refused, and clustered short options (`-vc f`) are not detected.
- [ ] Credential variables that bypass Claude Code and Codex isolation (SP2 design D8) are passed through
      unchanged; decide in `doctor` (SP5) whether to warn.
- [ ] Setting `[agents.codex] executable` to the npm-vendored `codex.exe` bypasses the npm launcher, which may put
      bundled tools such as `rg` on `PATH`; measure in a sandbox before recommending it.
- [ ] Creating the Aider config file needs a no-replace rename or hard links; a filesystem with neither fails
      with exit 4. macOS smbfs, msdos and exfat are not measured.
- [ ] A `.com` beside a `.exe` in one `PATH` directory is ignored although `cmd.exe` would prefer it.
- [ ] The Unix directory-sync failure branch of the Aider file writer is untested (like SP1 `config.rs` step 7a).

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
