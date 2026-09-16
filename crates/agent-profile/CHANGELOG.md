# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.2](https://github.com/ckir/aiprofiles/compare/v0.0.1...v0.0.2) - 2026-09-16

### Added

- repository resolution with link and unlink ([#19](https://github.com/ckir/aiprofiles/pull/19))

## [0.0.1](https://github.com/ckir/aiprofiles/releases/tag/v0.0.1) - 2026-09-15

### Added

- *(adapter)* adapter trait with Claude Code, Codex CLI and Aider
- *(exe)* skip relative PATH entries and refuse Windows shims with a hint
- *(adapter)* declarative adapter metadata types

### Fixed

- *(output)* redact dotted keys whose segments contain hyphens
- *(output)* redact dotted keys, headers and no- options; never echo an unknown option's value
- *(output)* redact secret argument values in the report
- *(exe)* trailing spaces and dots do not hide a shell extension

### Other

- core and explicit launch ([#8](https://github.com/ckir/aiprofiles/pull/8))
- scaffold the agent-profile workspace ([#1](https://github.com/ckir/aiprofiles/pull/1))

### Tests

- pin the redaction chain boundary and setup error texts
- close the SP2 test-audit gaps; redact chained secret options
- *(adapter)* end-to-end launch per adapter
- *(adapter)* common adapter contract suite
