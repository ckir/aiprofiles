//! Amp CLI: `--settings-file <profile settings.json>` (SP4 design §8.4; §9 outcome 5 — the flag is
//! accepted, but the agent's own writes went to the default `.local/share/amp` and did not follow it).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    ConflictOption, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const FLAG: &str = "--settings-file";
const FILE_NAME: &str = "settings.json";

/// Every spelling of the option Amp launches with; `--settings-file` first, so it is the one reported.
/// Only this spelling was measured — the probe's candidate sweep probed file contents, not flag
/// spellings, so Q6 stays unanswered and no abbreviation may ride along unmeasured (tracked debt).
const SETTINGS_OPTION: ConflictOption = ConflictOption { long: &[FLAG], short: None };

/// A zero-byte `settings.json`: the smallest content the agent accepts (candidate sweep: the empty
/// file and {} both exited 0) that sets no option, so it is what ships per §8.4 — a settings-file
/// adapter needs the file to exist before `amp` reads it.
pub const INITIAL_CONFIG: &[u8] = b"";

/// Shown on every launch.
pub const LAYERING_NOTE: &str = "--settings-file selects the user settings file, which workspace settings override and managed settings override both";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "amp",
    executable: "amp",
    mechanism: Mechanism::FlagFile(SETTINGS_OPTION),
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "amp-settings-file-v1",
        verified_at: "2026-10-17",
        upstream_version: "0.0.1792224035-gabd4f6",
        source_url: "https://ampcode.com/docs/cli/settings",
        notes: "measured in a sandbox at 0.0.1792224035-gabd4f6: a launch with --settings-file exits 0, \
                but the agent's own writes did not follow it — `accounts.json` and `device-id.json` were \
                written under the default `.local/share/amp` while the flag pointed at the probe's target \
                (§9 outcome 5), and the default `.config/amp` was `(absent)` at baseline and stayed so; \
                the candidate sweep accepted the empty file and {} (all exit 0); the strings scan found \
                AMP_API_KEY, AMP_URL and AMP_SETTINGS_FILE documented, and `--settings-file` itself does \
                not appear verbatim in the truncated help capture — the acceptance evidence is the \
                exit-0 use of the flag",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "measured: the flag was accepted (exit 0) but moved nothing to the target — \
                    `+ f /home/agent/.local/share/amp/device-id.json` was written at the default, and \
                    `.local/share/amp` was the only observed write target of the launch, while the \
                    target's `settings.json` was the probe's own file and stayed the only file there",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: no credential file was observed anywhere in the delta and no login step \
                    ran, so where stored credentials live was not established; `AMP_API_KEY` \
                    (strings: `documented AMP_API_KEY: present`) is the documented bypass variable",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "measured: `+ f /home/agent/.local/share/amp/accounts.json` and \
                    `+ f .../device-id.json` were written at the default while pointed at the flag's \
                    target — the device identity and account store do not move with `--settings-file`",
        },
    ],
    env: &[],
    // Empty on purpose. `--settings-file` is the mechanism's own option, and `check_conflicts` scans
    // that alongside this list, so naming it here again refused nothing extra. No abbreviated spelling
    // was measured (Q6 unanswered), so there is no list behind it either.
    conflicts: &[],
};

/// The Amp adapter.
#[derive(Debug, Clone, Copy)]
pub struct Amp;

impl Adapter for Amp {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        let dir = profile_dir(root, profile, METADATA.id);
        let file = dir.join(FILE_NAME);
        vec![(dir, PathKind::Dir), (file, PathKind::File { contents: INITIAL_CONFIG })]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        let mut planned = METADATA.mechanism.plan(self, ctx)?;
        planned.notes.push(LAYERING_NOTE.to_owned());
        Ok(planned)
    }
}
