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
- [ ] Creating the Aider config file needs a no-replace rename (Linux falls back to hard links; macOS needs
      exclusive-rename support, with no fallback). A filesystem without it fails with exit 4; macOS smbfs, msdos and
      exfat are not measured.
- [ ] A `.com` beside a `.exe` in one `PATH` directory is ignored although `cmd.exe` would prefer it, and a `.com`
      alone on `PATH` is reported as needing a shell although it is a native program.
- [ ] The Unix directory-sync failure branch of the Aider file writer is untested (like SP1 `config.rs` step 7a),
      and so is its temp-file `sync_all` before the rename (durability needs crash injection to observe).
- [ ] Report argument redaction is a name rule: it misses secrets passed positionally, in inline JSON, in
      `key=value` or `Name: value` forms without a sensitive name part, or under an option abbreviation without
      the sensitive part, or in an option name whose sensitive part is split by an invalid UTF-8 byte (the rule
      matches the lossy text), and it hides harmless values such as `--map-tokens 1024`.
- [ ] `ArgumentConflict` echoes the whole matched argument; redact it before any conflict option can carry a secret.
- [ ] The Codex "new profile starts logged out" note keys on the home directory being absent, so a present but
      empty home gives no note.
- [ ] An agent with no native executable cannot be launched on Windows until its vendor ships one.
- [ ] Adapter evidence is static; mechanism drift detection belongs to `doctor` (SP5).

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
