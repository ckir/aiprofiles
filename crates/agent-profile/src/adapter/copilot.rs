//! GitHub Copilot CLI: `COPILOT_HOME` per profile (SP2 design §8, probe outcome 5).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "COPILOT_HOME";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "copilot",
    executable: "copilot",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Experimental,
    evidence: AdapterEvidence {
        mechanism_id: "copilot-home-v1",
        verified_at: "2026-10-17",
        upstream_version: "1.0.98",
        source_url: "https://docs.github.com/en/copilot/reference/copilot-cli-reference/cli-config-dir-reference",
        notes: "the token is in the artefacts (`documented COPILOT_HOME: present`; help documents \
                the `~/.copilot` default via `--log-dir`) but the behaviour delta was empty: \
                `(no change at either watched location)` at both the target and the default \
                `~/.copilot (absent)` (§9 outcome 5), so the states below are undetermined; \
                the runtime cache the first use extracts at `.cache/copilot` is version-keyed, \
                not configuration or credentials (same call as opencode's `.cache` directory); \
                `COPILOT_GITHUB_TOKEN`, `GH_TOKEN` and `GITHUB_TOKEN` are the documented \
                credential bypass variables (strings scan: `documented GH_TOKEN: present`, \
                `documented GITHUB_TOKEN: present`, `documented COPILOT_GITHUB_TOKEN: ABSENT`)",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: `delta-env-COPILOT_HOME: (no change: nothing was created, moved \
                    or removed at the watched locations)` while the default `~/.copilot` was \
                    `(absent)` (§9 outcome 5, Unknown branch)",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: credential storage not located (§9 outcome 5); \
                    `COPILOT_GITHUB_TOKEN`, `GH_TOKEN` and `GITHUB_TOKEN` are the documented \
                    bypass variables (copilot.sh precedence), and no stored-credential \
                    observation was made",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the same delta — `(no change: nothing was created, moved or \
                    removed at the watched locations)` at both the target and the default \
                    `~/.copilot (absent)` (§9 outcome 5)",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The GitHub Copilot CLI adapter.
#[derive(Debug, Clone, Copy)]
pub struct Copilot;

impl Adapter for Copilot {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        // The measured delta was empty (§9 outcome 5), so nothing was observed following
        // `COPILOT_HOME` — whether the agent nests anything under the directory is undetermined,
        // and the adapter therefore declares only the profile directory itself.
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
