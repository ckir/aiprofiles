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
