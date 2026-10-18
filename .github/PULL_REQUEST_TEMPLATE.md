<!--
The PR title becomes the squash commit on `main` (see CONTRIBUTING.md, "Commit messages");
the `Conventional PR title` check enforces its form and release-plz writes the changelog
from it. `type(scope): description` — for example `feat(launch): prefer the agent-specific
repository mapping`.
-->

## What and why

<!-- One or two paragraphs: what this changes and why. Link the issue with "Closes #N" if
there is one. -->

## Specification reference

<!-- V3 is the specification this repository implements. Name the section(s) this change
addresses or follows (CONTRIBUTING.md, "Pull request process", item 4). For a design-backed
sub-project, link the design under docs/superpowers/ as well. -->

## Testing

<!-- What did you run, and what did it show? The local gate is:

     just check    # fmt + clippy + typos + shellcheck + line-citations + test + probe-tests + doc checks

CI runs the same on Linux, macOS and Windows and must be green on all three. -->

- [ ] `just check` passes locally
- [ ] New behaviour has tests; fixed behaviour has a regression test

## Documentation

- [ ] No user-visible behaviour changed, **or** the docs that describe the changed behaviour are updated in this PR

## Adapter or evidence changes

<!-- Delete this section if the PR touches neither. Rationale: docs/evidence/README.md —
a claim about isolation has to be checkable by someone who does not trust us. -->

- [ ] Every transcript is committed **byte-for-byte** from its CI artifact — never hand-edited
- [ ] The transcript and its registration (`REAL_ADAPTERS`, contract and end-to-end rows) land in the **same commit**
- [ ] Every new `basis` starts with `measured:`, `cited:` or `unmeasured:` — and `unmeasured:` exactly when the state is `Unknown`
- [ ] No adapter claims `CredentialIsolation: Supported` without an authenticated measurement
- [ ] Every README table row carries the support level **and** all three capability states — a support level alone overclaims (design §6.1)
- [ ] `cargo nextest run -E 'test(gate_a) or test(every_file_in_docs_evidence)'` is green
