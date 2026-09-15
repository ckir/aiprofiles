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

## SP3 known limits

Design: [docs/superpowers/specs/2026-09-15-sp3-resolution-design.md](docs/superpowers/specs/2026-09-15-sp3-resolution-design.md) §10.

- [ ] Git layouts that rely on `core.worktree`, `GIT_DIR` or `GIT_WORK_TREE` are not honoured; a bare repository is
      not a repository for resolution. `--repo` is the explicit override.
- [ ] A moved repository's mapping stays under the old path until it is linked again; the SP5 `repositories`
      report shows the orphan.
- [ ] A repository root that is not valid UTF-8 cannot be linked. On a Linux case-insensitive mount two letter-case
      spellings of one directory are two repository identities.
- [ ] Profile names in `config.toml` are validated with the host's rules, so a name Windows forbids makes a synced
      configuration invalid on Windows.
- [ ] An agent mapping for an agent this build does not know cannot be removed with `<agent> unlink`; SP5's
      `delete` refusal must name the key and field to edit.
- [ ] Shared multi-user machines (revisit with `doctor` in SP5, with a `safe.directory`-style escape hatch):
      discovery does not check who owns a `.git`, and a local user who swaps a checked file for a FIFO can block it.
- [ ] Unix automount paths and Windows mapped drive letters in a `gitdir`/`commondir` are not detected as network
      paths. On Windows the network refusal also refuses a repository on a volume without a drive letter, a local
      worktree of a repository on a share, and a share reached through two server spellings.
- [ ] A mapping made in the main checkout does not apply in its linked worktrees.
- [ ] A `link` or `unlink` that changes `config.toml` rewrites it with LF line endings and without a byte-order
      mark (`toml_edit` renders that way); a command that changes nothing leaves the file untouched.
- [ ] A stdout write that fails after `link` or `unlink` already changed `config.toml` exits 1 (`Io`), so the change
      is not visible in the output or the exit code; `<agent> resolve` with no profile loses its exit 4 the same way.
      Decide whether a write failure after a committed change deserves its own exit code.
- [ ] A hand-written `[repositories.'\\?\C:\…']` key is accepted but never matches the canonical `C:\…` root
      (`Path` treats the verbatim prefix as a different component), and `link` then adds a second entry for the same
      directory. Consider refusing verbatim keys, or comparing keys through `repo::strip_verbatim`.
- [ ] Discovery does not stop at a filesystem boundary the way `git` does, so a directory on a mount inside a
      checkout resolves to the enclosing repository (design §10). Decide with `doctor` (SP5) whether to warn.
- [ ] `unlink --repo <p>` can remove a different repository's live mapping when `<p>`'s meaning changed since `link`
      stored the key (design §10). Consider refusing when a later candidate key also matches an entry.
- [ ] A top-level `unlink` that removes the repository profile while agent mappings remain prints only `unlinked …`;
      the "agent mappings remain" note appears on the next run. Consider showing it on the run that creates the state.

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

- [ ] `windows_console` `breakaway_follows_a_controlled_caller_job` fails under plain `cargo test` (exit 125,
      "cannot spawn the sleeper: Access is denied") but passes under `cargo nextest`, the gate: it assumes one process
      per test. Isolate it (for example a nextest-only marker or a child process) or document it.

- [ ] Pin the actions in `.github/workflows/ci.yml` (`actions/checkout@v7`, `dtolnay/rust-toolchain@stable`,
      `Swatinem/rust-cache@v2`, `EmbarkStudios/cargo-deny-action@v2`, `taiki-e/install-action@nextest`) to commit SHAs
      with version comments, as the release and sandbox workflows already are.

- [ ] `sandbox/run.sh` picks the nextest download from the host `uname -m`; a Docker Desktop configured to build
      for another platform (`DOCKER_DEFAULT_PLATFORM`, Rosetta x86_64 default) gets the wrong binary. Pass the
      engine's build architecture instead if that setup is needed.

## Scaffold follow-ups

- [ ] Run `lefthook install` in each clone
