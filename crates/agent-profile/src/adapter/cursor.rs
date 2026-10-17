//! Cursor Agent CLI: `CURSOR_CONFIG_DIR` per profile (SP4 design §8, table row; probe outcome 1).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "CURSOR_CONFIG_DIR";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "cursor",
    executable: "cursor-agent",
    mechanism: Mechanism::Env(VAR),
    support: SupportLevel::Experimental,
    evidence: AdapterEvidence {
        mechanism_id: "cursor-config-dir-v1",
        verified_at: "2026-10-17",
        upstream_version: "2026.10.16-e3cb207",
        source_url: "https://cursor.com/docs/cli/reference/configuration",
        notes: "the agent's one pre-network write moved under CURSOR_CONFIG_DIR \
                (`+ f /home/agent/probe-target/cli-config.json`) while the default \
                `~/.cursor/cli-config.json` and the watched XDG fallback `.config/cursor` \
                recorded nothing (§9 outcome 1); the probe installed by the vendor script, \
                which refuses a version argument (§12), so upstream_version is the \
                `version-extracted` value; `CURSOR_API_KEY` is the documented bypass variable \
                (help: `--api-key <key> ... (can also use CURSOR_API_KEY env var)`); no \
                login/auth step ran and no credential-bearing file appeared",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "measured: `+ f /home/agent/probe-target/cli-config.json` — the agent's one \
                    pre-network write followed CURSOR_CONFIG_DIR into the target, and the watched \
                    fallback location recorded `f /home/agent/.config/cursor  (absent)`, so the \
                    XDG fallback was not chosen",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: where stored credentials live was not established (Q4) — no \
                    login/auth step ran and no credential-bearing file surfaced anywhere in the \
                    delta; `CURSOR_API_KEY` (strings: `documented CURSOR_API_KEY: present`) is \
                    the documented bypass variable, and no stored-credential observation was made",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the agent wrote only `cli-config.json` before it needed network \
                    access, so no session/state file was observable — the delta tail names \
                    nothing beyond `+ f /home/agent/probe-target/cli-config.json`",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Cursor Agent CLI adapter.
#[derive(Debug, Clone, Copy)]
pub struct Cursor;

impl Adapter for Cursor {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        // `CURSOR_CONFIG_DIR` names the configuration directory itself: the measured delta shows
        // `cli-config.json` written directly inside the target, so the profile directory IS the
        // config directory (the same call as opencode's). The agent's other writes stay at their
        // defaults and are deliberately not declared.
        vec![(profile_dir(root, profile, METADATA.id), PathKind::Dir)]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
