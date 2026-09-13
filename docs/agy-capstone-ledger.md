# AGY-CAPSTONE ledger

One row per completed capstone review of a committed implementation. In each review:
- an independent peer (agy) tore down the code a plan produced;
- every finding was verified by measurement before it was folded;
- the owner adjudicated GREEN.

The evidence column names things a reader can check: fold commits and the review transcript.

| Date | Range reviewed | Rounds | Verdict | Evidence | Discarded or deferred (anti-sweep) |
|---|---|---|---|---|---|
| 2026-09-13 | `99422d4..0e668d3`, excluding `Cargo.lock` and `docs/superpowers/` (SP0 scaffold; branch `sp0-scaffold`) | 2 | GREEN, owner-confirmed at `0e668d3` | No capstone folds were needed; the pre-capstone code-review fold is `dfc50b7` (fixture tests isolated from the inherited `FAKE_AGENT_EXIT`/`FAKE_AGENT_ECHO_ENV`). Transcript: agy cascade `07220325-ce4e-44cc-8887-889ff6dff33c`. Gate at `0e668d3`: `just check` passed on a clean build (8/8 tests); `cargo deny` ok; `cargo +1.85 check` ok; docs build with `-D warnings` ok; actionlint 0 errors. | Four blocking claims were rejected by measurement. (1) `actions/checkout@v7` does not exist: the `refs/tags/v7` tag exists and flux CI is green with it. (2) Dependabot auto-merge needs `pull_request_target`: GitHub's official recipe is `on: pull_request` with write permissions. (3) Reserving exit 125 blocks a 125 passthrough test: `FAKE_AGENT_EXIT=125` exits 125 with JSON on stdout, while a fixture error has empty stdout. (4) The stub's catch-all argument forces an SP1 rewrite: this is by design, since design §3.2 (lines 98–99) assigns the grammar to SP1. Linux and macOS first-run behaviour was reasoned, not measured; the PR run in plan Task 9 Step 1 measures it. |
