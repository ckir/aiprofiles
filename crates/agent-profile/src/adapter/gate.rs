//! The evidence gates (SP4 design §5), as predicates over `AdapterMetadata`.
//!
//! These live in the library rather than in `tests/adapter_contract.rs` for one reason: a gate whose only
//! expression is an assertion over `static METADATA` items cannot be proven non-vacuous. `cargo mutants`
//! mutates functions, not statics, so a weaker-than-intended gate — Gate B's biconditional written as a
//! single implication, say — would pass purely because no shipped adapter exhibits the excluded
//! combination. As functions they take negative fixtures, and a mutant reaches them.

use super::metadata::{AdapterMetadata, Capability, CapabilityState, SupportLevel};

/// The provenance prefixes a `basis` may start with (SP4 design D5).
pub const MEASURED: &str = "measured: ";
pub const CITED: &str = "cited: ";
pub const UNMEASURED: &str = "unmeasured: ";

/// The encoding of a failed probe in `evidence.upstream_version` (SP4 design §9 outcome 2).
pub const UNKNOWN_VERSION: &str = "unknown";

/// Why a gate refused. One variant per rule, so a test names the rule it is pinning.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GateFailure {
    /// Gate A: `source_url` is neither a URL nor the literal `measured`.
    SourceUrlShape { id: &'static str, source_url: &'static str },
    /// Gate A: `source_url` is `measured` but `notes` is empty.
    MeasuredWithoutNotes { id: &'static str },
    /// Gate A: `upstream_version` is outside `[A-Za-z0-9._-]+`, so it cannot name one evidence file.
    VersionCharset { id: &'static str, version: &'static str },
    /// Gate B: a `basis` carries no provenance prefix.
    BasisPrefix { id: &'static str, capability: Capability, basis: &'static str },
    /// Gate B: a `basis` is only its provenance prefix, so it cites a provenance for nothing.
    BasisWithoutContent { id: &'static str, capability: Capability, basis: &'static str },
    /// Gate B: a `basis` contains a newline, which would render as two report lines.
    BasisNewline { id: &'static str, capability: Capability },
    /// Gate B: `unmeasured:` without `Unknown`, or `Unknown` without `unmeasured:`.
    UnmeasuredMismatch { id: &'static str, capability: Capability, state: CapabilityState },
    /// Gate C: `Proven` with empty `notes`.
    ProvenWithoutNotes { id: &'static str },
    /// Gate C: `Proven` with more than one `Unknown` claim.
    ProvenWithTooManyUnknowns { id: &'static str, unknowns: usize },
    /// Gate C: a failed probe that did not degrade to `Experimental` with an unknown config claim.
    UnknownVersionNotExperimental { id: &'static str },
}

/// Gate A's shape half: what can be checked before any transcript exists (SP4 design §5.2).
///
/// The transcript clauses are deliberately absent. They assert files the probe run produces, and the probe
/// run happens after the phase that adds this function, so asserting them here would leave the branch red
/// for three adapters with no artefact able to make it green.
pub fn gate_a_shape(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    let evidence = metadata.evidence;
    let url =
        evidence.source_url.starts_with("https://") || evidence.source_url.starts_with("http://");
    if !url && evidence.source_url != "measured" {
        return Err(GateFailure::SourceUrlShape {
            id: metadata.id,
            source_url: evidence.source_url,
        });
    }
    if evidence.source_url == "measured" && evidence.notes.is_empty() {
        return Err(GateFailure::MeasuredWithoutNotes { id: metadata.id });
    }
    if evidence.upstream_version.is_empty()
        || !evidence
            .upstream_version
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(GateFailure::VersionCharset {
            id: metadata.id,
            version: evidence.upstream_version,
        });
    }
    Ok(())
}

/// Gate B: every `basis` carries a provenance prefix, no newline, and `unmeasured:` exactly when `Unknown`.
///
/// The biconditional is what makes D7 mechanical. Without it an implementer reaches `Proven` by writing
/// `NotGuaranteed` with an `unmeasured:` basis, and every other gate still passes.
pub fn gate_b(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    for claim in metadata.capabilities {
        let unmeasured = claim.basis.starts_with(UNMEASURED);
        let prefix = if unmeasured {
            Some(UNMEASURED)
        } else if claim.basis.starts_with(MEASURED) {
            Some(MEASURED)
        } else if claim.basis.starts_with(CITED) {
            Some(CITED)
        } else {
            None
        };
        let Some(prefix) = prefix else {
            return Err(GateFailure::BasisPrefix {
                id: metadata.id,
                capability: claim.capability,
                basis: claim.basis,
            });
        };
        if claim.basis[prefix.len()..].trim().is_empty() {
            return Err(GateFailure::BasisWithoutContent {
                id: metadata.id,
                capability: claim.capability,
                basis: claim.basis,
            });
        }
        if claim.basis.contains('\n') {
            return Err(GateFailure::BasisNewline {
                id: metadata.id,
                capability: claim.capability,
            });
        }
        if unmeasured != (claim.state == CapabilityState::Unknown) {
            return Err(GateFailure::UnmeasuredMismatch {
                id: metadata.id,
                capability: claim.capability,
                state: claim.state,
            });
        }
    }
    Ok(())
}

/// Gate C: what `Proven` costs, and what a failed probe forces (SP4 design §5, D6).
pub fn gate_c(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    let unknowns = metadata
        .capabilities
        .iter()
        .filter(|claim| claim.state == CapabilityState::Unknown)
        .count();
    if metadata.support == SupportLevel::Proven {
        if metadata.evidence.notes.is_empty() {
            return Err(GateFailure::ProvenWithoutNotes { id: metadata.id });
        }
        // V3:132-133 permits `Proven` while ONE capability remains unresolved; a second means the
        // mechanism itself is not understood, which is what `Experimental` is for.
        if unknowns > 1 {
            return Err(GateFailure::ProvenWithTooManyUnknowns { id: metadata.id, unknowns });
        }
    }
    // A failed probe observed nothing, so it cannot support a claim about the mechanism. Without this the
    // token exemption in Gate A becomes a hole: `upstream_version: "unknown"` beside a `Supported` config
    // claim would reach `Proven` with no mechanism token observed anywhere.
    // Case-insensitive: Gate A's charset ([A-Za-z0-9._-]) accepts "UNKNOWN"/"Unknown" as well as the
    // lowercase sentinel, and a cased spelling must degrade exactly like the lowercase one. This is
    // deliberately NOT mirrored in any shell script: the harness only ever *writes* the sentinel
    // (sandbox/probes/common.sh via `${probe_extracted:-unknown}`, sandbox/transcript.sh's
    // `version=unknown` default), never compares it — sandbox/verify-transcripts.sh's conclusion rule
    // reads probe-exit, not the version — so the harness cannot produce a cased variant. The only
    // reachable path to a cased sentinel is a maintainer hand-typing it into Rust metadata, which this
    // comparison alone must catch.
    if metadata.evidence.upstream_version.eq_ignore_ascii_case(UNKNOWN_VERSION) {
        let config_unknown = metadata.capabilities.iter().any(|claim| {
            claim.capability == Capability::ConfigIsolation
                && claim.state == CapabilityState::Unknown
        });
        if metadata.support != SupportLevel::Experimental || !config_unknown {
            return Err(GateFailure::UnknownVersionNotExperimental { id: metadata.id });
        }
    }
    Ok(())
}

/// Every gate that applies before the probe run, for one adapter.
pub fn gates_before_transcripts(metadata: &AdapterMetadata) -> Result<(), GateFailure> {
    gate_a_shape(metadata)?;
    gate_b(metadata)?;
    gate_c(metadata)
}
