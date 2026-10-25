// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

//! Kiro CLI: `KIRO_HOME` per profile (SP4 design §8.5 — settings move, the data tree does not).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "KIRO_HOME";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "kiro",
    executable: "kiro-cli",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "kiro-home-v1",
        verified_at: "2026-10-25",
        upstream_version: "2.34.0",
        source_url: "https://kiro.dev/cli/",
        notes: "the settings subsystem obeyed the variable — `+ f \
                /home/agent/probe-target/settings/cli.json` at the target against an \
                `(absent)` default — while a data tree appeared \
                under `.local/share/kiro-cli` (§9 outcome 5, Q3 answered partial); \
                the probe installed by the vendor script, which refuses a version \
                argument (§12), so upstream_version is the `version-extracted` value; \
                KIRO_API_KEY is the documented bypass variable and AWS_ACCESS_KEY_ID, \
                AWS_PROFILE are also strings-measured; no credential file appeared in \
                the delta",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "measured: `+ f /home/agent/probe-target/settings/cli.json` — the settings \
                    subsystem moved its file under KIRO_HOME, and the watched default `~/.kiro` \
                    and `.local/share/kiro-cli` were both `(absent)` at baseline, so the \
                    settings write at the target is the variable's",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: naming the vendor session required — no credential file appeared \
                    in the delta, `KIRO_API_KEY` (strings measured) is the documented bypass \
                    variable, and `AWS_ACCESS_KEY_ID`, `AWS_PROFILE` are also strings-measured",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "measured: `+ f /home/agent/.local/share/kiro-cli/data.sqlite3`, \
                    `+ d .../run-receipts` (`.reaper.lock`) and \
                    `+ f .../telemetry-export-drops.lock` — the data subsystem ignores KIRO_HOME \
                    and wrote the whole `.local/share/kiro-cli` tree at the default (Q3)",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Kiro CLI adapter.
#[derive(Debug, Clone, Copy)]
pub struct Kiro;

impl Adapter for Kiro {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        // `KIRO_HOME` names the directory the agent builds its tree under: the measured delta shows
        // `settings/cli.json` directly inside it, so the profile directory IS the home. The data
        // tree at `.local/share/kiro-cli` is the subsystem that ignores the variable (its basis) —
        // declaring it would create a directory the agent refuses to relocate.
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
