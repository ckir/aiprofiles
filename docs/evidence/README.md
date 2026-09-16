# Evidence transcripts

Every capability an adapter claims is backed by a file in this directory. A transcript is the record of
one probe run: what was installed, what version it reported, what its help text said, and — the part that
matters — what moved on disk when the agent's own isolation mechanism was applied.

The claims live in the adapter's `AdapterEvidence`; the measurement lives here. A `basis` beginning
`measured:` means a file in this directory shows it.

## Why these files exist at all

An adapter could simply assert that `FOO_HOME` isolates configuration. Nothing in the code would
contradict it, the tests would pass, and a user would have no way to tell a measured claim from a
plausible one. Isolation is the product's whole promise, so a claim about it has to be checkable by
someone who does not trust us.

That is also why the format is deliberately dull. A transcript is not a report written about a
measurement; it is the measurement's own output, bounded and stripped of terminal control sequences, with
a header saying where it came from.

## Naming

```
<id>-<version>.md              the CI transcript for that adapter at that upstream version
<id>-<version>-credentials.md  an off-CI transcript for an authenticated measurement
```

`<version>` is the adapter's `evidence.upstream_version`, and the two must agree: the contract suite
resolves this path from the registry, so a mismatch fails the build rather than going unnoticed.

A probe that could not install or run the agent records `unknown` as the version. That is not a gap — it
is a finding, and the gates force the adapter to `Experimental` because of it.

## The two custody shapes

The first line says which, and it is the only thing that distinguishes them at a glance.

**`custody: ci`** — produced by the Sandbox workflow. It carries the run id, the run URL and the commit
of the harness that produced it. Everything an adapter claims about configuration or state isolation
rests on one of these.

**`custody: off-ci`** — produced in a disposable sandbox on a maintainer's machine, carrying the sandbox
and its version, the host platform, the date, and who performed it. It exists for one reason: proving
that stored credentials separate requires logging in to each vendor, and an authenticated measurement
cannot run in CI. It is weaker evidence, it says so on its first line, and it can never be the only
backing for a configuration or state claim.

## How a CI transcript is verified

The `Evidence` job in `ci.yml` runs on every pull request that changes a file here. For each one it:

1. takes the agent id and version **from the registry**, never from the filename — `upstream_version`
   permits `-`, so `cursor-1.0.0-beta.1.md` splits two ways and the wrong split would verify the wrong
   file;
2. reads `run-id:` and `harness-commit:` out of the committed file;
3. checks the run belongs to this repository, is the Sandbox workflow, ran at that harness commit, and
   that the commit is an ancestor of the pull request's head;
4. checks the matrix job for *this agent* concluded as expected — `success`, or `failure` for an
   `unknown` transcript, because a probe that legitimately could not install is exactly what outcome 2
   records;
5. downloads that run's `sandbox-transcript-<id>` artifact and requires the committed file to be
   **byte-identical** to it.

Step 3 is not ceremony. Without it, someone with push access could dispatch the workflow on a throwaway
branch carrying an edited probe script that prints whatever they liked: the run would be genuinely green,
the artifact genuine, the bytes identical — and the branch that forged it would never appear in the pull
request.

Step 5 is why **no one edits a transcript**. If an excerpt is wrong, the probe script is wrong; fix the
script and run it again.

## Producing one

1. Actions → Sandbox → Run workflow, mode `probe`, and name the agents (or `all`).
2. Download the `sandbox-transcript-<id>` artifact from that run.
3. Read it. This is the review step, and it is the only one a machine cannot do: the job proves the bytes
   came from that run, not that the probe captured the right thing.
4. Commit it unchanged, and update the adapter's `upstream_version` to match.

Do all of this while the artifact still exists. Verification **fails closed** on an expired artifact, so a
transcript cannot be introduced or modified once its artifact has gone — the run has to be repeated.
Artifacts are retained for 90 days for exactly this reason.

## Refreshing one

A re-measurement **replaces** the file it supersedes; it does not sit beside it. One `custody: ci`
transcript per adapter is live at a time — the one the current `upstream_version` resolves to — and the
gates fail if an orphan is left behind. Git history holds the old one, which is where superseded evidence
belongs.

The exception is an `off-ci` credentials transcript, which sits alongside the CI one rather than
replacing it: it answers a different question, and the CI transcript is still the backing for every other
claim.

An `unknown` transcript from a failed probe is **deleted** once the agent is probed successfully. Leaving
it would be a durable false statement that the agent cannot be installed.

## What a transcript cannot tell you

It proves where files landed. It does not prove nothing leaked: a delta showing the configuration
directory moved cannot show that no credential was read from a shared location. State isolation is
measured only for agents that write history before they need the network, and credential isolation is not
measured in CI at all.

Evidence is also a snapshot — one version, one day. A mechanism can change upstream without this
repository noticing, and nothing here detects that drift.
