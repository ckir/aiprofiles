# AGY-TEST-AUDIT ledger

One row per completed test-coverage audit, run after AGY-CAPSTONE was GREEN over the same range. In each
audit:
- an independent peer (agy) audited the committed test suites for gaps a regression could slip through;
- every claimed gap was verified by measurement (a logic mutant that leaves the suite green);
- the owner scoped which gaps to close;
- every closing test was proven non-vacuous by the same mutant turning that test red.

Conventions: the newest row goes at the BOTTOM, and every `|` inside a cell is escaped as `\|`.

| Date | Audited range | Rounds | Verdict | Evidence |
| --- | --- | --- | --- | --- |
| 2026-09-14 | `99422d4..8209292` (peer audited to `f343722`; `8209292` is the fold. Files: smoke.rs, support/mod.rs, src/bin/fake-agent.rs, src/main.rs) | 1 | GAPS FOUND: 6 claimed, 6 confirmed by mutant, all 6 FOLDED (owner: close all) | Fold commit `8209292` (8 to 12 tests; 7 mutants, each red on its target test); brief `.clavity/seams/sp0-test-audit.md` |
| 2026-09-14 | `5220080..81a67a5` (SP1; peers audited to `da27060`; `81a67a5` is the fold. Two subagent auditors at the owner's direction: config/CLI/output suites and launcher/adapter/console suites) | 1 | GAPS FOUND: 8 claimed, 8 confirmed by the auditors' mutants or probes, all 8 FOLDED (owner: close all) | Fold commit `81a67a5` (102 to 108 tests): lock timeout; check 5 before check 6; bare-invocation help on stderr; non-UTF-8 `AGENT_PROFILE_HOME`; dry run of an existing profile; breakaway under a controlled caller job; inherited ignore-Ctrl-C (console-driver `CONSOLE_DRIVER_JOB_LIMITS`/`CONSOLE_DRIVER_IGNORE_CTRL_C` modes). Driver re-proved all 9 mutants (`scratchpad/audit_mutants.py`: lock timeout 1 s, check swap, help to stdout, UTF-8 unwrap, `profile_dir_exists=false`, breakaway always-allow and never-allow, fixture breakaway flag dropped, wrapper clearing the ignore attribute), each failing its new test. Brief `scratchpad/test-audit-brief.md`. Discarded below the floor: the parent-environment-unchanged assertion cannot fail by construction; job-before-spawn ordering race. |
