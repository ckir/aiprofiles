# Security Policy

## Supported versions

Agent Profile is pre-release. Until 0.1.0 ships there is no supported version and no backported
fixes — security work lands on `main`.

| Version | Supported |
|---|---|
| `main` (pre-0.1) | ✅ |

## Reporting a vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Report privately through GitHub's private vulnerability reporting: go to the
[Security tab](https://github.com/ckir/aiprofiles/security/advisories) of this repository and choose
**Report a vulnerability**. That opens a private advisory visible only to the maintainers.

Please include:

- A description of the issue and the impact you believe it has
- Steps to reproduce, ideally a minimal one
- The OS, the coding agent and its version, and the Agent Profile version
- Any relevant `--json` or `--dry-run` output, with secrets and private paths redacted

You can expect an acknowledgement within a few days. Please give us a reasonable window to ship a fix
before disclosing publicly.

## What counts as a vulnerability in Agent Profile

Agent Profile selects and launches coding-agent profiles. Its security boundary is set by spec §1 and
§36: it must never handle credentials, leak secrets, or let a repository choose a profile on its own.
Reports we especially want:

- **Credential handling.** Any path where Agent Profile copies OAuth credentials, extracts tokens,
  creates or stores credentials, or migrates credentials between profiles.
- **Secret disclosure.** Tokens, authorization headers, secret environment values or credential
  contents appearing in human output, `--json` output, `--dry-run`, `--verbose`, `doctor` or logs.
- **Repository-controlled selection.** A file or setting inside a repository that causes a profile to
  be selected or launched without the user explicitly linking it.
- **Escaping the profile root.** `delete` (or any other operation) removing or writing files outside
  `<application-root>/profiles/`, via crafted profile names, symlinks, junctions, reparse points or
  Windows device names.
- **Configuration corruption.** A crash, race or interrupted write that leaves `config.toml` partially
  written, or that silently replaces invalid configuration with defaults.
- **Shell mediation.** An agent launch that passes through `cmd.exe`, PowerShell or a Unix shell, so
  that arguments or profile values can be interpreted as shell syntax.
- **Hidden network activity.** Any network request or telemetry made by Agent Profile itself.

Crashes and panics on malformed input are bugs — please file them as normal issues unless they lead
to one of the outcomes above.

## Scope

Out of scope:
- The coding agents' own authentication, credential storage and network activity. That is the
  agent's responsibility (spec §36).
- Vulnerabilities in dependencies (report those upstream, though please tell us so we can pin or drop
  them).
- Issues that require an attacker who already has write access to the user's Agent Profile
  application root.
