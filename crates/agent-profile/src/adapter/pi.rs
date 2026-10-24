// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

//! Pi: `PI_CODING_AGENT_DIR` per profile (SP4 design §8 — provider keys bypass, unless `auth.json` is
//! present).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "PI_CODING_AGENT_DIR";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "pi",
    executable: "pi",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "pi-coding-agent-dir-v1",
        verified_at: "2026-10-17",
        upstream_version: "1.3.0",
        source_url: "measured",
        notes: "the baseline held auth.json and models-store.json at the default .pi/agent and the \
                same files moved under PI_CODING_AGENT_DIR with nothing left behind — the variable \
                names the config directory itself; the behaviour step's exit 1 is the probe's \
                PROBE_EXPECT (`pi -p hello` initialises before failing without a key); \
                ANTHROPIC_API_KEY, OPENAI_API_KEY and PI_CODING_AGENT_DIR are strings measured, \
                and the help lists PI_CODING_AGENT_SESSION_DIR, PI_PACKAGE_DIR, PI_OFFLINE, \
                PI_TELEMETRY and PI_SHARE_VIEWER_URL",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "measured: `+ f /home/agent/probe-target/auth.json` and \
                    `+ f /home/agent/probe-target/models-store.json` — the config directory's \
                    files moved **without** a `.pi/agent` prefix, so `PI_CODING_AGENT_DIR` names the \
                    config directory itself (the delta at the watched default `.pi` was empty)",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: `+ f /home/agent/probe-target/auth.json` under the variable against \
                    baseline `f /home/agent/.pi/agent/auth.json` at the default — pi stores \
                    credentials in `auth.json` inside its own config directory, which is exactly \
                    what `PI_CODING_AGENT_DIR` moves (§6's wider exception: the probe established \
                    where credentials are stored), so they travel with the config directory and no \
                    separate credential store exists to point anywhere else; env keys \
                    (`ANTHROPIC_API_KEY`, …) bypass the file entirely (help)",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Supported,
            basis: "measured: `+ d /home/agent/probe-target/sessions` and \
                    `+ f /home/agent/probe-target/models-store.json` — the session and \
                    models-store partial-measure cells appeared under the mechanism, and pi wrote \
                    plenty before it needed the network",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Pi adapter.
#[derive(Debug, Clone, Copy)]
pub struct Pi;

impl Adapter for Pi {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        // `PI_CODING_AGENT_DIR` names the config directory itself: the probe's files landed in the
        // target with no `.pi/agent` prefix, unlike the default `$PROBE_AGENT_HOME/.pi/agent`. The
        // profile directory IS the config directory, so nothing else is declared.
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
