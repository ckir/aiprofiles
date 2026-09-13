# TODO

Near-term work. Sub-project scope lives in [ROADMAP.md](ROADMAP.md).

## SP1 open decisions

- [ ] **Windows Ctrl-C mechanism** (spec §23.2, §24). Verify against Microsoft's documentation whether
      `SetConsoleCtrlHandler(NULL, TRUE)` is inherited by child processes. If it is, using it would make
      the launched agent ignore Ctrl-C. Candidate mechanism: a handler routine in the wrapper plus a job
      object with `JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE` for the no-orphan requirement. The observable
      behaviour in §24 is the oracle.
- [ ] **Application root** (spec §7). Choose the platform-appropriate location (and whether to use a
      crate for it), keeping it injectable for tests.
- [ ] **Configuration lock and atomic replace** (spec §18). Choose the file-lock mechanism, and how to get
      a guaranteed atomic replace on Windows; §18.1 forbids a silent non-atomic fallback.
- [ ] **Structured error types** and their mapping to the §33 exit codes.

## SP3 open decisions

- [ ] **Git repository discovery** (spec §13, §14). §14.2 requires Git's worktree metadata, so a plain
      upward walk for `.git` that ignores that metadata is not enough. Cover submodules, worktrees, nested
      repositories, symlinks and canonicalization failure.

## Housekeeping

- [ ] Enable GitHub private vulnerability reporting (see [SECURITY.md](SECURITY.md)). Branch
      protection is applied during SP0 (design §4.1).
- [ ] The MSRV (1.85) is not checked in CI. SP0 verified it by hand with
      `cargo +1.85 check --workspace --all-targets`. Decide whether to add a CI job.

## Scaffold follow-ups

- [ ] Run `lefthook install` in each clone
