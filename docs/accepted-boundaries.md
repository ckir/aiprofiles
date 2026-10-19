# Accepted test boundaries

The do-not-re-raise list of the AGY-TEST-AUDIT discipline: behaviours deliberately not pinned by a test,
each with the compensation that makes the gap acceptable and a code anchor for it. A future audit
re-validates that the compensation still exists before honouring an entry; an entry whose compensation
vanished is a live gap again. Owner-deferred gaps do not live here (they ride `.clavity/local-anomalies.md`
to a tracked `ROADMAP.md` item); this file holds accepted boundaries only. Newest entries go at the bottom of
their section; closed entries are removed.

## Profile lifecycle (SP5a)

- **`delete`'s final canonical-containment check** (`crates/agent-profile/src/lifecycle.rs`,
  `delete_target`, the `if !resolved.starts_with(&resolved_profiles)` refusal). Disabling it leaves the suite
  green because no reachable state gets past the guards in front of it: the target entry's own symlink
  refusal and the parent's symlink refusal (both in `delete_target`, pinned by
  `delete_refuses_symlinks_inside_and_outside_the_root_and_touches_nothing` in `tests/lifecycle.rs`), and
  `ProfileName` / `AgentId` validation, which admits no path separator. A symlinked `<root>/profiles`
  canonicalizes both sides of the comparison the same way. Pinning it would need a source seam, which the
  capstone GREEN at `e9d400e` does not cover. Accepted 2026-10-19 (test audit round 1, owner-scoped).
