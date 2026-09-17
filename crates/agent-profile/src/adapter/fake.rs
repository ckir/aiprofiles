//! The debug-only test agent: `FAKE_AGENT_HOME` per profile (SP1 design D1, SP2 design §5.4).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    ConflictOption, EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel,
    env_dir_plan, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "FAKE_AGENT_HOME";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "fake",
    executable: "fake-agent",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Experimental,
    evidence: AdapterEvidence {
        mechanism_id: "fake-home-v1",
        verified_at: "2026-09-15",
        upstream_version: "0.0.0",
        source_url: "measured",
        notes: "test fixture",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: a fixture has nothing to isolate",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: a fixture has nothing to isolate",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: a fixture has nothing to isolate",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[ConflictOption { long: &["--fake-profile"], short: None }],
};

/// The test agent, launched through the `fake-agent` fixture.
#[derive(Debug, Clone, Copy)]
pub struct Fake;

impl Adapter for Fake {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        env_dir_plan(self, ctx, VAR)
    }
}
