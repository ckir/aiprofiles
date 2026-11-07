# Landing page for agent-profile — design

Date: 2026-11-04. Status: draft for owner approval.

## 1. Intent and why

A single landing page on GitHub Pages whose only job is to get a developer who already uses coding agents from
"what is this" to running `agent-profile` on their own machine in about thirty seconds. Install and demo sit
above the fold. Everything deeper links to the README and the rustdoc rather than being rewritten.

The page also states, once and plainly, that the tool is officially supported by the
[clavity](https://github.com/ckir/clavity) project and works standalone without it.

## 2. Goals, non-goals, anti-goals

Goals

- A visitor on any of the five released targets copies one block, runs it, and sees the tool list their own
  installed coding agents with evidence rows. Measured on Linux x86_64 against the live v0.1.2 release:
  1.3 s warm, 1.8 s cold, without the checksum step.
- The headline block verifies the archive's SHA256 before anything is unpacked or run, with no manual step.
- The page is honest about what it cannot prove (unsigned binaries, OS prompts) and never claims more than
  the README does.
- The page itself is privacy-clean: no analytics, no third-party requests, no fonts or scripts from a CDN.

Non-goals

- Replacing the README or the rustdoc. The page links to both.
- A docs site, a blog, a changelog page, or multiple pages.
- Code signing or notarization. Out of scope; the page says the binary is unsigned where it matters.
- Package managers (Homebrew tap, Scoop, winget). Possible later; not part of this project.
- An interactive in-browser demo (WASM). The demo is a captured real transcript.

Anti-goals (what would make it a failure even if it technically works)

- A `curl | sh` installer. It contradicts the tool's trust pitch and was rejected in the consult.
- A snippet that runs the binary before the checksum has been checked.
- A snippet that silently fails on one of the five targets, or that asks the visitor to substitute a version.
- Any claim of speed on a platform where the snippet has not been timed.
- Fake or hand-edited terminal output presented as real.

## 3. Constraints found in the repo

- GitHub Pages is already in use: `.github/workflows/docs.yml` deploys rustdoc to the Pages root on every
  push to `main`. The landing page takes the root; rustdoc moves to `/rustdoc/`.
- The crate is `publish = false`, so `cargo install agent-profile` does not exist. Binaries ship as five
  archives per release from `.github/workflows/release.yml`, each holding the bare binary at the archive root:
  `agent-profile-{x86_64,aarch64}-unknown-linux-gnu.tar.gz`, `agent-profile-{x86_64,aarch64}-apple-darwin.tar.gz`,
  `agent-profile-x86_64-pc-windows-msvc.zip`.
- release-plz publishes the GitHub release first and the binaries job runs after it, so
  `releases/latest/download/<asset>` is a dead link for a window after every release. Measured on v0.1.2:
  release published 06:00:52 UTC, first asset 06:01:38, last asset 06:02:24. About ninety seconds.
- No checksum assets exist on any release today.
- Repo gates that new files must pass (`just check` and CI): `addlicense -check` with `header.txt` on every
  file type it recognises (HTML, CSS, JS, YAML, shell included); `typos`; `check-action-pins.sh` (every
  `uses:` pinned to a 40-hex SHA with a `# <ref>` comment); `shellcheck -s sh` on `scripts/*.sh`; line
  citations in comments must resolve. Rust 1.98 / edition 2024.
- Licence: PolyForm Noncommercial 1.0.0. The footer names it.
- The tool prints no secret and sends no telemetry; the page must hold itself to the same standard.

## 4. Approaches considered

### 4a. Hosting (owner decided)

Options: GitHub Pages root with rustdoc under a subpath; a separate host; a second Pages site in another
repo. Owner chose GitHub Pages. Driver's call within that: the landing page owns the root, rustdoc moves to
`/rustdoc/`, both deployed by the one Docs workflow so there is one artifact and one deploy.

### 4b. Install story (agy-first consult + one negotiation round, converged)

Options named: (A) `cargo install --git` headline; (B) hosted `curl | sh` installer; (C) adopt cargo-dist;
(D) per-platform download buttons; (E) in-place terminal snippet, PATH second; (F) package managers.

Rejected: A (needs Rust 1.98 and a minutes-long build); B (trust mismatch, new attack surface); C (fights
release-plz, replaces working pinned workflows); D as headline (browser-to-terminal context switch, Gatekeeper
on browser downloads); F (later, not now).

Chosen: E with agy's refinement, the self-verifying snippet. Download archive and its `.sha256`, verify,
unpack in the current directory, run `./agent-profile agents`. The `&&` chain stops before `tar` if the
checksum fails. PATH placement is a separate second section. Checksums and the release race are fixed in the
pipeline as part of this project (owner decision).

What the consult did not fold: agy's claim that the checksum defends against a tampering proxy. It does not,
since the checksum comes from the same origin over the same connection. It defends against truncation and
corruption, which is still worth one line.

### 4c. How the page is built (driver's call; owner talked outcomes)

Options: hand-written static HTML + CSS + a few lines of JS; a static-site generator (Zola, mdBook); a JS
framework. Chosen: hand-written static files under `site/`, copied into the Pages artifact by the Docs
workflow. One page needs no generator, a generator adds a toolchain to pin and a template layer to learn, and
a framework contradicts the no-third-party rule. Build-time substitutions (version, agent table) are done by
a small POSIX `sh` script, which the repo already knows how to lint.

### 4d. Visual direction (owner picked from three mockups)

A, instrument panel (light, mono throughout); B, night console (deep navy, phosphor amber, glowing terminal
hero); C, sealed compartments (paper, ink, one orange seal, profiles drawn as rooms). Owner chose B.

### 4e. Page length (owner picked from two layouts)

Short page (hero, one three-column strip, clavity band, footer) versus story page (adds a resolve-order
section and the twelve-agent evidence table). Owner chose the short page.

## 5. The page, as decided

Alignment: left-aligned throughout; max content width about 1100 px; single column on narrow screens.

### 5.1 Header

Wordmark `agent-profile`, the version of the latest published release, and links: GitHub, README, Rustdoc,
Releases.

The version is read at build time from the GitHub API (`gh api repos/ckir/aiprofiles/releases/latest`,
with the workflow's own token), not from `Cargo.toml`. Reason: after release-plz merges its release PR,
`main` carries the new version for the minutes until the binaries finish and the release is un-drafted, and
for ever if a build leg fails. Reading the latest published release keeps the header consistent with what
the `releases/latest/download/` links serve. The call is made by the build, not by the visitor's browser,
so the page's no-third-party rule is untouched. Only `docs.yml` makes the call; it passes the version to
the substitution script as an argument, so the script and `just check` stay offline-clean and a local
preview can pass any string. If the API call fails the deploy fails; the page never guesses a version.

### 5.2 Hero

- Headline: "Your coding agents, in separate rooms."
- Lead: "Work and personal. Client A and client B. One profile per repository, chosen for you when you
  launch, and the agent never sees across the wall."
- The terminal window, glowing, is the hero element and the one place the design spends its boldness. It
  carries five tabs, one per released archive, so the visitor never guesses an architecture and the snippet
  never shells out to detect one:

  | Tab | Asset |
  |---|---|
  | Linux x86_64 | agent-profile-x86_64-unknown-linux-gnu.tar.gz |
  | Linux arm64 | agent-profile-aarch64-unknown-linux-gnu.tar.gz |
  | macOS Apple Silicon | agent-profile-aarch64-apple-darwin.tar.gz |
  | macOS Intel | agent-profile-x86_64-apple-darwin.tar.gz |
  | Windows x86_64 | agent-profile-x86_64-pc-windows-msvc.zip |

- Linux snippet (shown for x86_64; arm64 differs only in the asset name):

  ```sh
  curl -fsSLO https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-unknown-linux-gnu.tar.gz \
   && curl -fsSL  https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256 | sha256sum -c - \
   && tar -xzf agent-profile-x86_64-unknown-linux-gnu.tar.gz \
   && ./agent-profile agents
  ```

- macOS snippet: same shape with `shasum -a 256 -c -` in place of `sha256sum -c -`.
- Windows snippet (PowerShell 5.1 and 7): `Invoke-WebRequest` for the zip and the `.sha256`, compare
  `(Get-FileHash).Hash` to the first field of the `.sha256` file case-insensitively, `Expand-Archive -Force`
  (a second run must not fail because the first left `agent-profile.exe` behind), then
  `.\agent-profile.exe agents`. PowerShell 5.1 has no `&&`, so the whole snippet is one script block
  invoked as `& { ... }` whose first statements are `$ErrorActionPreference = 'Stop'` and
  `$ProgressPreference = 'SilentlyContinue'`; the block keeps both preference changes out of the visitor's
  session. The comparison must be null-guarded (`if ($hash -and $expected -and $hash -eq $expected)`), because `$null -eq $null` is
  true and a failed download would otherwise run whatever binary an earlier attempt left behind. The exact
  text is the builder's, pinned by the site-check test in §7.
- A copy button per tab. Without JS, all five blocks are visible and stacked; the page is complete without
  scripts.
- Beside the snippet on wide screens and below it on narrow ones, the demo: the real output of
  `agent-profile agents`, captured inside the project's sandbox container (`sandbox/run.sh`), where the
  agents are installed and the paths carry no personal username. The capture is committed verbatim as
  `site/demo/agents.txt`, with no header or comment lines, so a byte prefix of it is exactly what the tool
  printed. Its version and date live beside it in `site/demo/agents.meta` (two lines, `version=` and
  `date=`). The page shows the first three agents in the binary's registry order, which is claude, codex,
  aider (`crates/agent-profile/src/adapter/mod.rs`, `REAL_ADAPTERS`), and ends with a line `… and 9 more`
  that the page adds, not the capture. The caption reads "Captured in the
  project's sandbox from agent-profile <version> on <date>" with both values taken from the meta file. Not generated at build time, since a CI runner has no agents installed and
  the output would be empty. A check in §7 fails if the shown lines are not a prefix of the committed
  capture.
- Under the snippet, one plain line: "Downloads one archive from the GitHub release, checks its SHA256,
  unpacks one file into this folder and runs it. The archive and the binary stay in this folder until you
  delete them or keep the binary (below). Nothing is installed anywhere else. Nothing leaves your machine."
- Hero copy says "Try it in seconds." The page does not quote a number of seconds on any tab until that
  platform has been timed (§7).

### 5.3 Strip of three

- Keep it. One command per OS to move the binary onto PATH (`~/.local/bin` on Linux and macOS, a user
  `bin` folder on Windows). What macOS and Windows may ask when a binary is unsigned, with the exact unblock
  command (`xattr -d com.apple.quarantine agent-profile`; `Unblock-File`), stated as what may happen, not
  as a promise either way. `cargo install --git https://github.com/ckir/aiprofiles agent-profile` for Rust
  users, with the 1.98 requirement.
- Downloads. The five archives for the latest release, each with its `.sha256` link beside it, plus a link to
  all releases.
- What it never does. Never copies credentials, never extracts tokens, never sends telemetry, never trusts the
  repository to choose a profile. Each line links to the README section that backs it.

### 5.4 Clavity band

"Officially supported by clavity, and complete on its own." One sentence on what clavity is (Claude Code
paired with Antigravity), one on standalone use, two links: clavity, README.

### 5.5 Footer

Licence name with link, GitHub, README, Rustdoc, Releases.

### 5.6 Look

Direction B. Palette: page `#0E1424` to `#0A0F1C` radial, terminal `#070B14`, text `#E9E6DC`, muted
`#9AA3B8`, accent amber `#FFB040`, terminal text `#FFD58A`, success `#7CE7B0`. Type: Space Grotesk for
words, JetBrains Mono for anything typed or printed. Both are OFL-licensed and are vendored as the
upstream-published woff2 files under `site/fonts/` with their licence files; no subsetting toolchain is
added. No font request leaves the origin. Motion: one blinking cursor in
the terminal and the glow; nothing else moves on its own; `prefers-reduced-motion` stops the cursor. Visible
keyboard focus on tabs, copy buttons and links. The snippet lines are longer than a 360 px viewport, so each
scrollable snippet container carries `tabindex="0"` and an `aria-label`, otherwise a keyboard user cannot
scroll it sideways. Contrast meets WCAG AA on every text colour over its background (measured, §10).
Responsive to 360 px wide.

## 6. Pipeline changes (in scope)

### 6.1 Checksums and the release race

- `release-plz.toml`: `git_release_draft = true`. A draft release is not "latest", so the
  `releases/latest/download/` URL keeps pointing at the previous complete release until the new one is
  finished.
- `release.yml`: a new `finalize` job after the five-leg build matrix. It downloads every archive of the
  release, writes one `<asset>.sha256` per archive in `sha256sum` format (`<hex>  <filename>`, two spaces),
  uploads them with `--clobber`, and only then runs `gh release edit <tag> --draft=false`. If any build leg
  fails the release stays draft and the latest link stays on the previous release.
- The Windows archive's checksum is computed in the Linux `finalize` job after download, so the format is
  identical for all five.
- The finalize logic lives in `scripts/release-finalize.sh` (POSIX sh, shellcheck-clean, licence header)
  with the signature `release-finalize.sh <tag> <directory>`: it writes one `.sha256` beside every file in
  `<directory>` matching `agent-profile-*.tar.gz` or `agent-profile-*.zip` and ignores everything else, so
  a dispatch re-run that downloads the previous run's `.sha256` files alongside the archives neither hashes
  them nor fails on them. The job calls it as `scripts/release-finalize.sh "$TAG" .` after `gh release download`;
  the test under `scripts/tests/` calls it with a fixture directory of fake archives and checks the
  `.sha256` format byte for byte. The job itself only downloads, calls the script, uploads and un-drafts.
- After the un-draft, the finalize job runs `gh workflow run docs.yml` so the site rebuilds with the new
  version in its header. `workflow_dispatch` is one of the two event types GitHub does fire from a
  `GITHUB_TOKEN`-driven action, so this needs no extra token, but the dispatch API needs `actions: write`,
  which neither `release.yml` (`contents: write`) nor the `binaries` job in `release-plz.yml` grants today.
  The finalize job gets `contents: write` and `actions: write`; the build legs stay at `contents: write`.
  Because `release.yml` is a reusable workflow called from the `binaries` job in `release-plz.yml`, and a
  called workflow cannot hold more permission than its caller grants, the `binaries` job's permissions
  block also gains `actions: write`. The manual `workflow_dispatch` path of `release.yml` carries its own
  top-level permissions and needs the same addition there.
  If the dispatch fails after the un-draft, the release is already live and the page is stale until the
  next push to `main`; the job prints the manual command (`gh workflow run docs.yml`) in its failure
  output so the operator can finish it without rebuilding anything.
- `git_release_draft` is a real option (release-plz `crates/release_plz/src/config.rs`,
  `pub git_release_draft: Option<bool>`, shipped in 0.3.25 on 2023-09-24; the action pins 0.3.167). It is
  separate from `git_release_type`, whose values are `prod`, `pre` and `auto`; the existing `prod` line
  stays.
- Evidence that drafts fit the existing wiring, read from release-plz's source (main, 2026-11-04,
  `crates/release_plz_core/src/command/release.rs`): the tag is created and pushed before the GitHub
  release is created (`create_git_tag_and_release`), the draft flag flows into that release request
  (`git/forge.rs`, `draft: &release_info.draft`), and the next run skips any package whose tag already
  exists (`release_package_if_needed`, "Already published - Tag ... already exists"). So the tag the
  binaries job checks out exists, `releases_created` and `.[0].tag` are still produced, and a draft is never
  released twice. The builder confirms the same three points against the tag the pinned action
  (`release-plz/action` v0.5.145) ships before merging.
- Operator note: if any build leg fails, the release stays a draft, the `releases/latest` links keep serving
  the previous release, and the page keeps the previous version. The failed workflow run is the signal.
  Re-running `release.yml` by dispatch with the same tag finishes it. Abandoning it instead leaves the tag
  in place, and release-plz skips any version whose tag exists, so an abandoned draft is never released by
  automation: delete the draft release and its tag before merging the next release PR, or the version is
  simply skipped.
- Testable now: `workflow_dispatch` of `release.yml` with tag `v0.1.2` re-uploads the archives, adds the five
  checksum files, and the un-draft is a no-op on an already published release.
- Sequencing gate: the page must not go live before a published release carries the five `.sha256`
  assets, because every headline snippet fetches one. Order of landing: pipeline change, then the dispatch
  on v0.1.2, then the site. The plan carries this as a hard dependency between tasks.

### 6.2 Docs workflow becomes the site workflow

- `docs.yml` builds rustdoc as today, then assembles one artifact: `site/` at the root (with build-time
  substitutions applied), `target/doc/` at `rustdoc/`, the existing generated workspace index at
  `rustdoc/index.html`. One upload, one deploy, permissions unchanged.
- Two redirect stubs at the old root paths, `agent_profile/index.html` and `fake_agent/index.html`, each a
  `<meta http-equiv="refresh">` to the same path under `rustdoc/`, so a link saved while rustdoc lived at
  the root still lands. Nothing in the repo links to the root today.
- Trigger stays push to `main` and manual dispatch; the finalize job's dispatch (§6.1) is the second.

## 7. Quality gates and oracles

- Everything under `site/` and any new script carries `header.txt` so `just license-check` passes.
  `addlicense` only inspects file types it knows (HTML, CSS, JS, YAML, shell among them) and skips `.txt`,
  `.md` and binary files, which is why the repo's Markdown files carry no header today and the gate
  passes; the vendored `OFL.txt` files and `.woff2` fonts therefore need no header and no new `-ignore`.
- A new `scripts/check-site.sh` with `just site-check`, added to `just check` and CI:
  - every asset name in the page's five snippets and the downloads grid exists in the `release.yml` matrix,
    and every matrix target has a tab (drift guard, the same idea as `gen-tool-table.sh --check`);
  - every snippet's download URL uses `releases/latest/download/`;
  - the page contains no `http(s)://` reference to any host other than `github.com`,
    `polyformproject.org` and the page's own origin (privacy guard);
  - the HTML has no `<script src=` or `<link href=` to a non-relative URL;
  - the demo lines shown in the page are a prefix of `site/demo/agents.txt`, so the transcript cannot be
    hand-edited without the gate noticing.
- A dispatch-only workflow, `.github/workflows/snippet-timing.yml`, never scheduled and never on push,
  that runs each tab's snippet verbatim (read from the page, not retyped) on `ubuntu-latest`,
  `macos-latest` and `windows-latest` and prints the wall-clock time. It is the oracle for three things
  only: the snippet's syntax on each shell, the download and checksum path, and the time those take. It
  proves nothing about what a desktop OS shows a human, because headless runners have no interactive
  session and never raise Gatekeeper or SmartScreen prompts. Results are recorded as a dated line per
  platform in this document's §10, marked "CI runner". The page ships before the Windows and macOS numbers
  exist, because it quotes no number; it may quote one only after that platform's line is in §10, and
  only phrased as transfer-and-run time, never as "works without prompts". The page's wording for OS
  prompts stays "may ask" until someone runs the snippet on a physical Mac and a physical Windows desktop
  and records the result in §10.
- Pinning test for the pipeline: after the first real release under the new workflow, the five `.sha256`
  assets exist and the release is not draft. Until then the dispatch run on v0.1.2 is the evidence.

## 8. Left to the builder

- Exact HTML structure, CSS, and the tab and copy-button script.
- The exact Windows snippet text, within §5.2's requirements.
- Wording of the Keep-it commands per OS.
- The build-time substitution script (`scripts/` is the convention; POSIX sh; shellcheck-clean).
- The internals of `scripts/check-site.sh`, within §7's list.

## 9. Calls made by the driver after the builder check

The builder check (a fresh reviewer reading this document) asked six questions. None needed the owner;
each is settled above and listed here so the owner can check them while reading:

1. Demo transcript: captured in the sandbox container, committed verbatim, prefix-checked (§5.2, §7).
2. Draft releases with the existing release-plz wiring: settled by reading release-plz's source (§6.1).
3. Finalize logic as a tested script under `scripts/`, wired into `just check` and `ci.yml` separately,
   since CI repeats each gate rather than calling `just check` (§6.1, §7).
4. Header version from the latest published release via the API at build time, and a docs rebuild
   dispatched by the finalize job (§5.1, §6.1).
5. Redirect stubs for the two old root-level rustdoc paths (§6.2).
6. The timing workflow is its own dispatch-only file; the page ships before its numbers exist (§7).

## 10. Measurements

| Date | Platform | Snippet | Result |
|---|---|---|---|
| 2026-11-04 | Linux x86_64 (this box, uncontrolled background load) | without checksum step, v0.1.2 | 1.8 s cold, 1.3 s warm |
| 2026-11-04 | v0.1.2 release timeline | release published 06:00:52Z, first asset 06:01:38Z, last 06:02:24Z | about 90 s dead-link window |
| 2026-11-04 | palette contrast (WCAG 2 formula) | every §5.6 text/background pair | lowest 5.41:1 (bar labels on terminal), highest 14.70:1; all pass AA for body text |
| 2026-11-04 | v0.1.2 checksum dispatch (run 41901626183, feat/landing-page) | finalize job: 5 `.sha256` assets attached, release not a draft; `sha256sum -c -` from a clean directory prints OK. First attempt: both macOS legs never got a hosted runner (GitHub capacity), `gh run rerun --failed` succeeded | pipeline proven on the live release |
| 2026-11-05 | CI runner ubuntu-latest x86_64 (snippet-timing run 41972667060, v0.1.2) | checksum-verifying snippet, `time sh` twice | 1.07 s cold, 0.56 s warm; steps passed, so the checksum matched |
| 2026-11-05 | CI runner ubuntu-24.04-arm aarch64 (same run) | same | 0.73 s cold, 0.24 s warm |
| 2026-11-05 | CI runner macos-latest aarch64 (same run) | same, `shasum -a 256 -c -` | 1.16 s cold, 0.52 s warm |
| 2026-11-05 | CI runner macos-15-intel x86_64 (same run) | same | 1.39 s cold, 0.37 s warm |
| 2026-11-05 | CI runner windows-latest, PowerShell 7 (same run) | Windows snippet, Stopwatch | 2.04 s cold, 0.20 s warm |
| 2026-11-05 | CI runner windows-latest, Windows PowerShell 5.1 (same run) | same | 1.19 s cold, 0.20 s warm. Hosted runners with uncontrolled load and a fast network; one pair of runs per leg, so a spread, not a point. Gatekeeper behaviour on a curl-downloaded binary is still unverified |

## Stand-downs

Findings stood down by the panel review, one line each with the reason; see the review ledger in
`.clavity/scratch/landing-panel/`.

- REJECTED: "`git_release_draft` is not a valid key; use `git_release_type = \"draft\"`" (agy round 1,
  Dependency Cynic). Measured in release-plz source: `config.rs` declares `pub git_release_draft:
  Option<bool>` and the `ReleaseType` enum has only `Prod`, `Pre`, `Auto`; the option shipped in 0.3.25.
- DISCARDED-BELOW-FLOOR: the site-check host allow-list could be bypassed by a script that builds a URL
  at runtime (solo round, Mechanism Gamer). The page's only script is the tab and copy-button code,
  committed and reviewed in the same PR; no code path builds a URL.
- UNVERIFIED (owner to accept or test): whether a `curl`-downloaded, unsigned binary run from Terminal
  triggers Gatekeeper on current macOS (agy round 1, Mechanism Gamer and Q2). `curl` sets no quarantine
  attribute, so the driver expects no prompt, but neither party can measure it here. The page's "may
  ask" wording covers both outcomes; a run on a physical Mac settles it (§7).
