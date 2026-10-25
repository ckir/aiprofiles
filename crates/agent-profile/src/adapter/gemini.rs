// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

//! Gemini CLI: `GEMINI_CLI_HOME` per profile (SP2 design §8.1).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "GEMINI_CLI_HOME";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "gemini",
    executable: "gemini",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "gemini-cli-home-v1",
        verified_at: "2026-10-25",
        upstream_version: "0.65.0",
        source_url: "https://geminicli.com/docs/cli/enterprise/",
        notes: "the sandbox delta moved config (projects.json) and history/tmp state into \
                <target>/.gemini while the default location gained nothing; \
                GEMINI_API_KEY, GOOGLE_API_KEY and GOOGLE_APPLICATION_CREDENTIALS are the \
                credential bypass variables (binary strings measured)",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "measured: `+ d /home/agent/probe-target/.gemini` and \
                    `+ f /home/agent/probe-target/.gemini/projects.json` — the variable moved the \
                    configuration inside the target, nested one level as the agent creates `.gemini` \
                    itself",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: separating stored credentials requires a vendor session; \
                    GEMINI_API_KEY, GOOGLE_API_KEY and GOOGLE_APPLICATION_CREDENTIALS \
                    (strings measured) are the documented bypass variables, and no \
                    stored-credential observation was made",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Supported,
            basis: "measured: history and tmp state moved as \
                    `+ d /home/agent/probe-target/.gemini/history` and \
                    `+ d /home/agent/probe-target/.gemini/tmp`",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Gemini CLI adapter.
#[derive(Debug, Clone, Copy)]
pub struct Gemini;

impl Adapter for Gemini {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        // `GEMINI_CLI_HOME` names a directory CONTAINING `.gemini` (probe and transcript): the agent
        // creates the nested `.gemini` itself, so the adapter declares only the profile directory.
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
