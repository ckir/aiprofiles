# Landing Page Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship the GitHub Pages landing page for agent-profile and the release-pipeline change it depends on, in the order the spec's sequencing gate requires.

**Architecture:** Static files under `site/` are assembled with rustdoc into one Pages artifact by the existing Docs workflow. The release workflow gains a finalize job that writes a `.sha256` per archive and un-drafts the release only after every binary is up. Three POSIX `sh` scripts under `scripts/` carry the logic so the repo's existing gates (shellcheck, licence headers, action pins) cover them, each with a test under `scripts/tests/`.

**Tech Stack:** Hand-written HTML, CSS and a few lines of JS; POSIX `sh`; GitHub Actions; `gh` CLI; release-plz 0.3.167.

**Spec:** `docs/superpowers/specs/2026-11-04-landing-page-design.md`

## Global Constraints

- Every new `.html`, `.css`, `.js`, `.sh` and `.yml` file carries `header.txt` verbatim as its first comment (`just license-check`). `.txt`, `.meta` and `.woff2` files carry none.
- Every new `uses:` is pinned to a 40-hex SHA with a `# <ref>` comment (`just action-pins`). Reuse the SHAs already in `.github/workflows/ci.yml` for checkout, rust-toolchain, rust-cache, upload-pages-artifact and deploy-pages.
- Scripts are POSIX `sh`, pass `shellcheck -s sh`, source `scripts/lib/scan-guard.sh` wherever a scan's empty result would otherwise read as clean, and resolve `repo_root` the way `scripts/gen-tool-table.sh:18` does.
- The page makes no request to any host other than its own origin and `github.com`; links may also point at `polyformproject.org`. Fonts are vendored.
- Commit types on this branch are `ci`, `docs`, `build`, `test` or `chore`, never `feat` or `fix`: `release-plz.toml` turns a `feat`/`fix` commit into a crate release and this work does not change the crate.
- Asset names are exactly the five in `.github/workflows/release.yml:44-58`: `agent-profile-<target>.tar.gz` for the four Unix targets and `agent-profile-x86_64-pc-windows-msvc.zip`.
- `.sha256` file content is `<64 lowercase hex>  <asset file name>\n` (two spaces), the `sha256sum` format, so `sha256sum -c -` and `shasum -a 256 -c -` both accept it.
- Order of landing (spec §6.1): Tasks 1 to 3 and the owner-run dispatch in Task 3 complete before the page (Task 7) can deploy.

## Review Focus

1. A visitor runs the Linux snippet twice in the same folder: the second run must overwrite the archive and binary and succeed. Pinned by the snippet-timing workflow running each snippet twice in one job (Task 8).
2. The checksum file and archive disagree: the chain must stop before `tar`/`Expand-Archive` and leave a clear `sha256sum: WARNING` or PowerShell error, never run a binary. Pinned by `scripts/tests/check-site.sh` asserting the Linux snippet has `sha256sum -c -` and both apple-darwin snippets have `shasum -a 256 -c -` (t10, t10b) before `tar` and the Windows block has the null-guarded compare before `Expand-Archive` (Task 5).
3. `Get-FileHash` returns uppercase hex and the `.sha256` holds lowercase: the compare must be case-insensitive. Pinned by the Windows leg of the snippet-timing workflow against real assets (Task 8).
4. The page with JavaScript disabled: all five snippet blocks visible and stacked, copy buttons absent or inert. Pinned by `scripts/tests/check-site.sh` asserting no snippet block carries `hidden` in the source HTML (Task 5).
5. A finalize dispatch re-run on a release that already carries `.sha256` files: the script must ignore them and rewrite the five checksums. Pinned by `scripts/tests/release-finalize.sh` (Task 1).

---

### Task 1: Checksum script

**Files:**
- Create: `scripts/release-finalize.sh`
- Create: `scripts/tests/release-finalize.sh`
- Modify: `justfile:113` (the `check` line) and add a recipe beside `action-pins` at `justfile:67-69`
- Modify: `.github/workflows/ci.yml:89-98` (add a sibling job after `action-pins`)

**Interfaces:**
- Produces: `sh scripts/release-finalize.sh <tag> <directory>`. For every file in `<directory>` matching `agent-profile-*.tar.gz` or `agent-profile-*.zip`, writes `<directory>/<file>.sha256` holding `<hex>  <file>\n`. Ignores every other file. Exits 1 with a message naming `<directory>` when no archive matches (the scan-guard class). Exits 2 on usage error. `<tag>` is echoed in the summary line only; it does not select files.

- [ ] **Step 1: Write the failing test** `scripts/tests/release-finalize.sh`, in the style of `scripts/tests/check-action-pins.sh:13-28` (`set -eu`, `root`, `check`, `tmp` with `trap`). Cases:

```sh
# fixture: five fake archives (tar.gz x4, zip x1) with known bytes, plus a stray
# agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256 holding "stale" and a README.txt
# t1 exit status 0
# t2 exactly five .sha256 files exist afterwards
# t3 content of agent-profile-x86_64-pc-windows-msvc.zip.sha256 equals
#    "$(sha256sum agent-profile-x86_64-pc-windows-msvc.zip | cut -d' ' -f1)  agent-profile-x86_64-pc-windows-msvc.zip"
# t4 the stale .sha256 was overwritten (no longer "stale")
# t5 no README.txt.sha256 and no *.sha256.sha256 exists
# t6 an empty directory exits 1 and prints the directory path
# t7 one argument exits 2
```

- [ ] **Step 2: Run it to verify it fails**

Run: `sh scripts/tests/release-finalize.sh`
Expected: FAIL lines naming `release-finalize.sh: not found` (or the equivalent), exit non-zero.

- [ ] **Step 3: Implement `scripts/release-finalize.sh`** with the signature above. Use `sha256sum` (GNU coreutils; the job runs on `ubuntu-latest`). Match with `find "$dir" -maxdepth 1 \( -name 'agent-profile-*.tar.gz' -o -name 'agent-profile-*.zip' \)` and count matches before the loop; zero matches is the exit-1 path.

- [ ] **Step 4: Run the test and shellcheck**

Run: `sh scripts/tests/release-finalize.sh && shellcheck -s sh scripts/release-finalize.sh scripts/tests/release-finalize.sh`
Expected: seven `ok   -` lines, no shellcheck output.

- [ ] **Step 5: Wire the gate.** Add to `justfile` a recipe `release-finalize-tests:` running `sh scripts/tests/release-finalize.sh`, append it to the `check:` line, and add a CI job `release-finalize-tests` modelled on `action-pins` (`.github/workflows/ci.yml:89-98`) that runs the same command. Add the licence header to both scripts.

Run: `just release-finalize-tests && just license-check && just action-pins && just shellcheck`
Expected: all exit 0.

- [ ] **Step 6: Commit**

```bash
git add scripts/release-finalize.sh scripts/tests/release-finalize.sh justfile .github/workflows/ci.yml
git commit -m "ci(release): checksum script for release archives"
```

---

### Task 2: Draft releases and the finalize job

**Files:**
- Modify: `release-plz.toml` (after line 28, `git_release_type = "prod"`)
- Modify: `.github/workflows/release.yml:29-30` (top-level permissions) and append a job after `build` (ends at line 96)
- Modify: `.github/workflows/release-plz.yml:65-66` (the `binaries` job permissions)

**Interfaces:**
- Consumes: `scripts/release-finalize.sh <tag> <directory>` (Task 1).
- Produces: every published release carries `<asset>.sha256` beside each of the five archives; a release is a draft until all five are uploaded; after un-draft the job dispatches `docs.yml`.

- [ ] **Step 1: `release-plz.toml`.** Add `git_release_draft = true` directly under `git_release_type = "prod"`, with a comment citing spec §6.1 (release-plz publishes the release before the binaries exist; the finalize job in release.yml un-drafts it).

- [ ] **Step 2: Permissions.** In `release.yml` change the top-level block to `contents: write` plus `actions: write`. In `release-plz.yml` give the `binaries` job `contents: write` and `actions: write`. Add a job-level `permissions: contents: write` block to the `build` job in `release.yml` (it has none today, `release.yml:36-40`) so its matrix legs do not inherit `actions: write`.

- [ ] **Step 3: Add the `finalize` job** to `release.yml`:

```yaml
  finalize:
    name: Checksums and publish
    needs: build
    runs-on: ubuntu-latest
    permissions:
      contents: write
      actions: write
    steps:
      # No `ref:` here on purpose: the job needs scripts/release-finalize.sh from the ref that RUNS this
      # workflow (main for a real release, the feature branch for Task 3's dispatch), not from the
      # release tag, whose tree predates the script. Only the build legs check out the tag.
      - uses: actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1 # v7.0.1
        with:
          persist-credentials: false
      - name: Download archives
        env: { GH_TOKEN: "${{ github.token }}", TAG: "${{ inputs.tag }}" }
        run: mkdir assets && gh release download "$TAG" --dir assets --pattern 'agent-profile-*' --repo "$GITHUB_REPOSITORY"
      - name: Write checksums
        run: sh scripts/release-finalize.sh "${{ inputs.tag }}" assets
      - name: Upload checksums
        env: { GH_TOKEN: "${{ github.token }}", TAG: "${{ inputs.tag }}" }
        run: gh release upload "$TAG" assets/*.sha256 --clobber --repo "$GITHUB_REPOSITORY"
      - name: Publish the release
        env: { GH_TOKEN: "${{ github.token }}", TAG: "${{ inputs.tag }}" }
        run: gh release edit "$TAG" --draft=false --latest --repo "$GITHUB_REPOSITORY"
      - name: Rebuild the site
        env: { GH_TOKEN: "${{ github.token }}" }
        run: gh workflow run docs.yml --repo "$GITHUB_REPOSITORY" || { echo "::error::site rebuild not dispatched; run: gh workflow run docs.yml"; exit 1; }
```

The `--pattern 'agent-profile-*'` download also fetches old `.sha256` files on a re-run; the script ignores them (Task 1, t4/t5).

- [ ] **Step 4: Verify the gates**

Run: `just action-pins && just typos && just license-check`
Expected: exit 0.

- [ ] **Step 5: Commit**

```bash
git add release-plz.toml .github/workflows/release.yml .github/workflows/release-plz.yml
git commit -m "ci(release): draft until binaries and checksums are up, then publish"
```

---

### Task 3: Prove the pipeline on v0.1.2 (owner-run)

**Files:** none changed.

**Interfaces:**
- Consumes: Task 2's workflow on this branch.
- Produces: the five `.sha256` assets on the live v0.1.2 release, which Task 7's page needs before it can deploy.

- [ ] **Step 1: Owner confirmation.** This step rebuilds and re-uploads the public v0.1.2 archives and adds checksum files to a live release. Stop and ask the owner before running it.

- [ ] **Step 2: Push and dispatch from the branch**

`feat/landing-page` exists only locally (`git branch -r` lists no `landing` ref), and a dispatch needs the ref and the Task 1-2 changes on GitHub. Pushing without a PR does not start `ci.yml` (its triggers are `main` only). The owner's Step 1 approval covers this push.

Run: `git push -u origin feat/landing-page && gh workflow run release.yml --ref feat/landing-page -f tag=v0.1.2 --repo ckir/aiprofiles && gh run watch --repo ckir/aiprofiles`
Expected: all five `build` legs and `finalize` green.

- [ ] **Step 3: Verify the assets**

Run: `gh release view v0.1.2 --repo ckir/aiprofiles --json assets,isDraft --jq '.isDraft, ([.assets[].name|select(endswith(".sha256"))]|length)'`
Expected: `false` then `5`.

- [ ] **Step 4: Verify a checksum end to end from a clean directory**

Run (in an empty scratch directory): `curl -fsSLO https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-unknown-linux-gnu.tar.gz && curl -fsSL https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256 | sha256sum -c -`
Expected: `agent-profile-x86_64-unknown-linux-gnu.tar.gz: OK`

- [ ] **Step 5: Record it** in spec §10 as a dated line and commit: `docs(specs): record the v0.1.2 checksum dispatch`.

---

### Task 4: Demo capture in the sandbox

**Files:**
- Create: `site/demo/agents.txt`
- Create: `site/demo/agents.meta`

**Interfaces:**
- Produces: `site/demo/agents.txt`, the verbatim stdout of `agent-profile agents` run inside the sandbox with claude, codex and aider installed, no header, no trailing edits. `site/demo/agents.meta` holds two lines, `version=<output of agent-profile --version, number only>` and `date=<YYYY-MM-DD>`.

- [ ] **Step 1: Open the sandbox with network**

Run: `sandbox/run.sh --net shell`
If the sandbox cannot start on this machine, STOP and report `STATE_MISMATCH: sandbox unavailable`; the owner supplies the capture.

- [ ] **Step 2: Inside the shell**, build the binary from the copied checkout (`cargo build --release --bin agent-profile`), install the three agents the way their probes do (`sandbox/probes/claude.sh:19` uses `probe_npm_install @anthropic-ai/claude-code`; follow `codex.sh` and `aider.sh` for the other two), then capture:

```sh
target/release/agent-profile agents > /out/agents.txt
target/release/agent-profile --version
```

`/out` is the only host-mounted path (`sandbox/run.sh:276`); the container and anything outside `/out` is deleted on exit (`run.sh:214-215`), so write there and copy `agents.txt` from the host share directory unchanged.

- [ ] **Step 3: Verify the capture** is raw: first line starts with `agent:        claude`, the third `agent:` block is `aider`, and the file contains no `/home/<your user>` path.

Run: `head -1 site/demo/agents.txt; grep -c '^agent:' site/demo/agents.txt; grep -n "^agent:" site/demo/agents.txt | sed -n 3p`
Expected: `agent:        claude`, `12`, a line ending in `aider`.

- [ ] **Step 4: Write `site/demo/agents.meta`** and commit

```bash
git add site/demo/agents.txt site/demo/agents.meta
git commit -m "docs(site): capture the agents demo in the sandbox"
```

---

### Task 5: Site gate

**Files:**
- Create: `scripts/check-site.sh`
- Create: `scripts/extract-snippet.sh`
- Create: `scripts/tests/check-site.sh`
- Modify: `justfile` (recipe `site-check`, appended to `check:`), `.github/workflows/ci.yml` (job `site-check` beside `action-pins`)

**Interfaces:**
- Consumes: `site/demo/agents.txt` (Task 4); `.github/workflows/release.yml:44-58` as the asset oracle.
- Produces:
  - `sh scripts/extract-snippet.sh <html-file> <target>` prints the text content of `<pre class="snippet" id="snip-<target>">…</pre>` with `&amp;`, `&lt;`, `&gt;`, `&quot;` decoded and tags stripped; exit 1 if the id is absent.
  - `sh scripts/check-site.sh [<site-dir>]` (default `site`) exits 0 when every rule below holds, otherwise prints one `check-site: <rule>: <detail>` line per failure and exits 1.
- HTML contract the page (Task 7) must satisfy, so the gate and the page agree:
  - five `<pre class="snippet" id="snip-<target>" tabindex="0" aria-label="…">` blocks, one per target in `release.yml`, none carrying the `hidden` attribute in the source;
  - a `<pre id="demo">` whose text content is the first N lines of `site/demo/agents.txt` followed by exactly one line `… and 9 more`, where N is the line count up to and excluding the blank line that precedes the fourth `agent:` line;
  - the literal `{{VERSION}}` once, in the header, and the literals `{{DEMO_VERSION}}` and `{{DEMO_DATE}}`
    once each, in the demo caption; `scripts/build-site.sh` (Task 7) fills all three.

- [ ] **Step 1: Write the failing test** `scripts/tests/check-site.sh`. Build fixtures as temp site dirs with a minimal `index.html` and a copy of `site/demo/`. Cases:

```sh
# t1 a conforming fixture passes (exit 0, no output)
# t2 a snippet whose asset name is agent-profile-x86_64-unknown-linux-musl.tar.gz fails: "asset not in release.yml"
# t3 only four snip- ids: fails naming the missing target
# t4 a download URL under releases/download/v0.1.2/ fails: "not releases/latest/download"
# t5 <link href="https://fonts.googleapis.com/..."> fails: "external host"
# t6 <script src="https://cdn.example/x.js"> fails: "external host"
# t7 a demo block whose second agent line is edited fails: "demo not a prefix"
# t8 a snip- block with the hidden attribute fails: "snippet hidden without JS"
# t9 missing {{VERSION}} fails
# t10 the Linux x86_64 snippet without "sha256sum -c -" before "tar -xzf" fails: "unverified unpack"
# t10b a macOS snippet (apple-darwin) without "shasum -a 256 -c -" before "tar -xzf" fails: "unverified unpack (macOS)"
# t11 the Windows snippet without "$ErrorActionPreference = 'Stop'" fails: "unguarded PowerShell"
# t12 extract-snippet decodes "&amp;&amp;" to "&&" (compare its output to a known string)
# t13 extract-snippet on an unknown target exits 1
```

- [ ] **Step 2: Run it to verify it fails**

Run: `sh scripts/tests/check-site.sh`
Expected: FAIL lines, scripts not found.

- [ ] **Step 3: Implement `scripts/extract-snippet.sh`** (sed/awk over the HTML; the page keeps each snippet's `<pre>` and `</pre>` on their own lines so a line-range extraction is enough) and **`scripts/check-site.sh`** implementing the rules in the Interfaces block. Read the target list from `release.yml` with `grep -E '^\s+target: ' | awk '{print $2}'` and assert the count is 5 before comparing (scan-guard). Allowed hosts: `github.com`, `polyformproject.org`.

- [ ] **Step 4: Run tests and shellcheck**

Run: `sh scripts/tests/check-site.sh && shellcheck -s sh scripts/check-site.sh scripts/extract-snippet.sh scripts/tests/check-site.sh`
Expected: thirteen `ok   -` lines, no shellcheck output.

- [ ] **Step 5: Wire `site-check`** into `justfile` (recipe plus `check:`) and `ci.yml` (job modelled on `action-pins`, running `sh scripts/check-site.sh` then `sh scripts/tests/check-site.sh`). Until Task 7 lands, `just site-check` fails because `site/index.html` does not exist; that is the expected red state between Tasks 5 and 7, so leave the `check:` wiring for Task 7's final step.

- [ ] **Step 6: Commit**

```bash
git add scripts/check-site.sh scripts/extract-snippet.sh scripts/tests/check-site.sh justfile .github/workflows/ci.yml
git commit -m "test(site): gate the landing page against the release matrix and the demo capture"
```

---

### Task 6: Fonts

**Files:**
- Create: `site/fonts/SpaceGrotesk[wght].woff2`, `site/fonts/SpaceGrotesk-OFL.txt`, `site/fonts/JetBrainsMono-Regular.woff2`, `site/fonts/JetBrainsMono-SemiBold.woff2`, `site/fonts/JetBrainsMono-OFL.txt`

**Interfaces:**
- Produces: the five files above, referenced by `site/site.css` in Task 7 with relative URLs.

- [ ] **Step 1: Fetch upstream files.** Space Grotesk: `fonts/woff2/SpaceGrotesk[wght].woff2` and `OFL.txt` from `github.com/floriankarsten/space-grotesk` at tag `2.0.0`. JetBrains Mono: from the `JetBrainsMono-2.304.zip` asset of release `v2.304` at `github.com/JetBrains/JetBrainsMono`, take `fonts/webfonts/JetBrainsMono-Regular.woff2`, `fonts/webfonts/JetBrainsMono-SemiBold.woff2` and `OFL.txt`. If the zip layout differs, use the files with those names wherever they sit and note the path in the commit message.

- [ ] **Step 2: Verify** each woff2 begins with the bytes `wOF2` and the licence files mention "SIL Open Font License".

Run: `for f in site/fonts/*.woff2; do head -c4 "$f"; echo " $f"; done; grep -l "SIL Open Font License" site/fonts/*OFL.txt`
Expected: `wOF2` before each path; both licence files listed.

- [ ] **Step 3: Commit**

```bash
git add site/fonts
git commit -m "build(site): vendor Space Grotesk and JetBrains Mono under the OFL"
```

---

### Task 7: The page

**Files:**
- Create: `site/index.html`, `site/site.css`, `site/site.js`
- Create: `scripts/build-site.sh`
- Modify: `.github/workflows/docs.yml:47-94` (index generation) and `:102-104` (upload path); `justfile` `check:` line

**Interfaces:**
- Consumes: the HTML contract in Task 5; `site/demo/*` (Task 4); `site/fonts/*` (Task 6).
- Produces: `sh scripts/build-site.sh <version> <out-dir>` copies `site/` into `<out-dir>`, replaces `{{VERSION}}` with `<version>` and `{{DEMO_VERSION}}` / `{{DEMO_DATE}}` with the `version=` / `date=` values of `site/demo/agents.meta`, writes `<out-dir>/agent_profile/index.html` and `<out-dir>/fake_agent/index.html` as `<meta http-equiv="refresh" content="0; url=../rustdoc/<same path>">` stubs; exits 2 without both arguments.

- [ ] **Step 1: Write the page** to the spec's §5 content and §5.6 look. Copy is fixed by the spec: headline "Your coding agents, in separate rooms.", the lead sentence, the five tab labels, the disclosure line, the "Keep it", downloads and "What it never does" columns, the clavity band, the footer. Snippets, exactly:

Linux x86_64 (the other three Unix tabs differ only in the asset name; macOS tabs use `shasum -a 256 -c -`):
```sh
curl -fsSLO https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-unknown-linux-gnu.tar.gz \
 && curl -fsSL  https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-unknown-linux-gnu.tar.gz.sha256 | sha256sum -c - \
 && tar -xzf agent-profile-x86_64-unknown-linux-gnu.tar.gz \
 && ./agent-profile agents
```

Windows:
```powershell
& {
  $ErrorActionPreference = 'Stop'; $ProgressPreference = 'SilentlyContinue'
  $u = 'https://github.com/ckir/aiprofiles/releases/latest/download/agent-profile-x86_64-pc-windows-msvc.zip'
  Invoke-WebRequest "$u" -OutFile agent-profile-x86_64-pc-windows-msvc.zip
  $expected = ((Invoke-WebRequest "$u.sha256").Content -split '\s+')[0]
  $hash = (Get-FileHash .\agent-profile-x86_64-pc-windows-msvc.zip -Algorithm SHA256).Hash
  if ($hash -and $expected -and ($hash -ieq $expected)) {
    Expand-Archive .\agent-profile-x86_64-pc-windows-msvc.zip -DestinationPath . -Force
    .\agent-profile.exe agents
  } else { throw "checksum mismatch: expected $expected got $hash" }
}
```

Page structure: header; hero with tabs (`role="tablist"`, buttons with `aria-selected`), the five `<pre class="snippet">` blocks, a copy button per block, the disclosure line, the `<pre id="demo">` with caption; the three-column strip; the clavity band; the footer. Without JS all five blocks render stacked under their labels; `site.js` only hides the non-selected ones and wires the copy buttons.

- [ ] **Step 2: Run the gate**

Run: `sh scripts/check-site.sh`
Expected: exit 0, no output.

- [ ] **Step 3: Write `scripts/build-site.sh`** with the signature above and a test case appended to `scripts/tests/check-site.sh`: `t14` builds into a temp dir and asserts `{{VERSION}}` is absent from the output, the given version is present, and both redirect stubs exist.

Run: `sh scripts/tests/check-site.sh`
Expected: fourteen `ok   -` lines.

- [ ] **Step 4: Rework `docs.yml`.** Keep the `cargo doc` step. Replace the "Generate workspace index" heredoc's target with `target/doc/index.html` as today, then add ONE step (a `run:` is its own shell, so a variable does not carry to the next step) that sets `VERSION=$(gh api repos/$GITHUB_REPOSITORY/releases/latest --jq .tag_name | sed 's/^v//')` (with `GH_TOKEN`) and runs `sh scripts/build-site.sh "$VERSION" site-dist`, `mkdir -p site-dist/rustdoc && cp -r target/doc/. site-dist/rustdoc/`, and change the upload `path:` to `site-dist`. The build job keeps `contents: read`; the API read needs nothing more.

- [ ] **Step 5: Local assembly check**

Run: `sh scripts/build-site.sh 0.0.0-local /tmp/site-dist && ls /tmp/site-dist/index.html /tmp/site-dist/agent_profile/index.html && grep -c '0.0.0-local' /tmp/site-dist/index.html`
Expected: both paths listed, `1`.

- [ ] **Step 6: Finish the gate wiring** (`site-check` into `just check`), then run the whole local gate.

Run: `just check`
Expected: exit 0.

- [ ] **Step 7: Commit**

```bash
git add site/index.html site/site.css site/site.js scripts/build-site.sh scripts/tests/check-site.sh .github/workflows/docs.yml justfile
git commit -m "build(site): landing page assembled with rustdoc into the Pages artifact"
```

---

### Task 8: Snippet timing workflow

**Files:**
- Create: `.github/workflows/snippet-timing.yml`

**Interfaces:**
- Consumes: `scripts/extract-snippet.sh` (Task 5); the live release assets (Task 3).
- Produces: a dispatch-only workflow whose job log prints, per target, the wall-clock seconds of the snippet run twice.

- [ ] **Step 1: Write the workflow.** `on: workflow_dispatch` only. Matrix of the five targets with their runners as in `release.yml:42-58`. Each job: checkout (pinned), `sh scripts/extract-snippet.sh site/index.html <target> > snippet` (give that step `shell: bash` so `sh` resolves on the Windows leg), then for Unix targets `time sh "$GITHUB_WORKSPACE/snippet"` twice from inside a fresh `mktemp -d`; for Windows a `pwsh` step that runs the snippet twice, each timed with `[System.Diagnostics.Stopwatch]::StartNew()` and `& ([scriptblock]::Create((Get-Content -Raw snippet)))` outside `Measure-Command`, so the `agent:` lines reach the log. Permissions `contents: read`.

- [ ] **Step 2: Verify the gates**

Run: `just action-pins && just license-check && just typos`
Expected: exit 0.

- [ ] **Step 3: Dispatch after merge.** GitHub accepts `workflow_dispatch` only for a workflow present on the default branch, so this runs in Task 9 once the PR is merged: `gh workflow run snippet-timing.yml --repo ckir/aiprofiles && gh run watch --repo ckir/aiprofiles`. Expected: five green jobs; each log shows the `agent:` lines and two timings. Until then the YAML is checked by `just action-pins` only.

- [ ] **Step 4: Record** (post-merge, in a follow-up `docs(specs)` commit) each platform's second (warm) timing as a dated "CI runner" line in spec §10 and commit: `docs(specs): record snippet timings from CI runners`.

- [ ] **Step 5: Commit the workflow**

```bash
git add .github/workflows/snippet-timing.yml
git commit -m "ci(site): dispatch-only snippet timing on the three OS images"
```

---

### Task 9: Branch finish

- [ ] **Step 1:** Run `just check` once more; expected exit 0.
- [ ] **Step 2:** Push and open the PR with a `build(site):` title, body linking the spec and Task 3's run URL.
- [ ] **Step 3:** After merge, confirm the deploy: `curl -fsSI https://ckir.github.io/aiprofiles/ | head -1` expects `HTTP/2 200`, and `curl -fsSI https://ckir.github.io/aiprofiles/rustdoc/agent_profile/index.html | head -1` expects `HTTP/2 200`.
- [ ] **Step 4:** After merge, run Task 8 Step 3 (dispatch `snippet-timing.yml`), then Task 8 Step 4 (record timings in a follow-up `docs(specs)` commit).
