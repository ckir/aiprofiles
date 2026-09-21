//! Codex CLI: `CODEX_HOME` per profile (SP2 design §5.2).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "CODEX_HOME";

/// Shown when the profile's home did not exist at plan time.
pub const NEW_PROFILE_NOTE: &str =
    "new profile starts logged out; run codex login with this profile";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "codex",
    executable: "codex",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "codex-home-v1",
        verified_at: "2026-09-15",
        upstream_version: "0.153.4",
        source_url: "measured",
        notes: "codex --help: -p/--profile layers $CODEX_HOME/<name>.config.toml; binary strings OPENAI_API_KEY, \
                CODEX_API_KEY, CODEX_ACCESS_TOKEN, CODEX_SQLITE_HOME; CODEX_HOME semantics per the Codex CLI \
                configuration documentation",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "cited: config.toml and <name>.config.toml live in CODEX_HOME; project-level \
                    configuration layering is not measured",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Conditional,
            basis: "measured: OPENAI_API_KEY, CODEX_API_KEY and CODEX_ACCESS_TOKEN bypass it; auth.json \
                    and the keyring key follow CODEX_HOME",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Conditional,
            basis: "measured: CODEX_SQLITE_HOME relocates the state database",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Codex CLI adapter.
#[derive(Debug, Clone, Copy)]
pub struct Codex;

impl Adapter for Codex {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        let mut planned = METADATA.mechanism.plan(self, ctx)?;
        if !planned.paths[0].existed {
            planned.notes.push(NEW_PROFILE_NOTE.to_owned());
        }
        Ok(planned)
    }
}
