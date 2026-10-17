//! Cline CLI: `CLINE_DATA_DIR` env var (SP4 design §8.3, §9 outcome 5 — the env var is honoured,
//! but the measured delta at the watched locations was empty; the agent wrote nothing).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "CLINE_DATA_DIR";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "cline",
    executable: "cline",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Experimental,
    evidence: AdapterEvidence {
        mechanism_id: "cline-cli-v1",
        verified_at: "2026-10-17",
        upstream_version: "3.0.73",
        source_url: "measured",
        notes: "measured in a sandbox at 3.0.73: --help lists `CLINE_DATA_DIR` as a Configuration \
                directory and `--config`/`--data-dir` as directory flags; a launch with the env var set \
                exits 0, but the delta at both watched locations (target and default) was \
                `(no change: nothing was created, moved or removed at the watched locations)` — the \
                agent writes nothing before it needs the network (§9 outcome 5); the strings scan \
                found ANTHROPIC_API_KEY, CLINE_API_KEY, OPENAI_API_KEY and OPENROUTER_API_KEY \
                documented; Q1 unanswered (no isolation or sandbox-mode side effects were observed \
                before the agent needed the network); three `Unknown`s ⇒ Experimental under Gate C/D6",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the probe's delta at both watched locations — the target and the default \
                    `/home/agent/.cline` — was `(no change: nothing was created, moved or removed at the \
                    watched locations)`, so whether the agent's own config writes would follow CLINE_DATA_DIR \
                    was not observable while the agent writes nothing before it needs the network \
                    (§9 outcome 5)",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: no login/auth step ran and no credential-bearing file surfaced anywhere \
                    in the delta, so where stored credentials live was not established; the documented \
                    bypass variables are `CLINE_API_KEY`, `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` \
                    and `OPENROUTER_API_KEY` (strings: `documented CLINE_API_KEY: present`)",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the agent's only pre-network write was `cli-node-extra-ca-certs.pem`, \
                    present at the default location at baseline, and the measured delta recorded \
                    `(no change: nothing was created, moved or removed at the watched locations)`, so no \
                    state write was observable to place (§9 outcome 5)",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

// The Cline adapter.
#[derive(Debug, Clone, Copy)]
pub struct Cline;

impl Adapter for Cline {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        // `CLINE_DATA_DIR` names a directory CONTAINING the agent's state (probe and transcript): the
        // agent does not create a nested subdirectory, so the adapter declares only the profile
        // directory itself.
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
