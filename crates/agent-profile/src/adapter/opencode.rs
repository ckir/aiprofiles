//! OpenCode: `OPENCODE_CONFIG_DIR` per profile (SP2 design §8.2).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "OPENCODE_CONFIG_DIR";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "opencode",
    executable: "opencode",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "opencode-config-dir-v1",
        verified_at: "2026-10-17",
        upstream_version: "v2.0.32",
        source_url: "https://opencode.ai/docs/cli/",
        notes: "the config-side service.json followed OPENCODE_CONFIG_DIR into the target while \
                session/state data (opencode.db with -shm/-wal, shell/, opencode.log) stayed at \
                the default .local/share/opencode (§9 outcome 1, both repetitions); \
                OPENCODE_CONFIG names a config file to READ and relocates none of the agent's \
                writes, so the mechanism is the directory variable alone; the probe's launches run \
                unauthenticated, so auth.json is never written and no credential file appears",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "measured: `+ f /home/agent/probe-target/service.json` — the config-side write \
                    followed OPENCODE_CONFIG_DIR into the target, and the second delta \
                    (`envfile:OPENCODE_CONFIG`) shows the same write at the default \
                    `.config/opencode/service.json` when only the file variable is set",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: `auth.json` lives under `XDG_DATA_HOME` (default \
                    `~/.local/share/opencode`, §8.2) — a directory neither mechanism moves (both \
                    deltas left the data tree in place and the strings scan contains no \
                    `OPENCODE_DATA*`/`OPENCODE_HOME` token) — so stored credentials sit outside \
                    the directory the config variable relocates; `ANTHROPIC_API_KEY` and \
                    `OPENAI_API_KEY` (strings measured) are the documented bypass variables",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: `+ f /home/agent/.local/share/opencode/opencode.db` (with `-shm`/\
                    `-wal`), `+ d .../shell/...` and `opencode.log` stayed at \
                    `.local/share/opencode` under both mechanisms: neither names a data directory \
                    and the strings scan contains no `OPENCODE_DATA*`/`OPENCODE_HOME` token (Q2)",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The OpenCode adapter.
#[derive(Debug, Clone, Copy)]
pub struct Opencode;

impl Adapter for Opencode {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        // `OPENCODE_CONFIG_DIR` names the configuration directory itself (the target received
        // `service.json` directly at its top level), so the profile directory IS the config
        // directory. Data stays where §8.2 and the measured delta recorded it — under
        // `XDG_DATA_HOME`, default `~/.local/share/opencode` — and is deliberately not declared.
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
