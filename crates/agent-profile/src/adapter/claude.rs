//! Claude Code: `CLAUDE_CONFIG_DIR` per profile (SP2 design §5.1).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    EnvOverride, PathKind, PlanContext, PlannedLaunch, SupportLevel, env_dir_plan, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const VAR: &str = "CLAUDE_CONFIG_DIR";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "claude",
    executable: "claude",
    mechanism_summary: "environment variable CLAUDE_CONFIG_DIR",
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "claude-config-dir-v1",
        verified_at: "2026-09-15",
        upstream_version: "2.1.270",
        source_url: "https://code.claude.com/docs/en/authentication",
        notes: "CLAUDE_CONFIG_DIR relocates .credentials.json and keys the macOS Keychain entry per directory; \
                ANTHROPIC_API_KEY, ANTHROPIC_AUTH_TOKEN, CLAUDE_CODE_OAUTH_TOKEN, CLAUDE_CODE_OAUTH_REFRESH_TOKEN, \
                ANTHROPIC_PROFILE and CLAUDE_CODE_USE_BEDROCK/VERTEX/FOUNDRY override it (binary strings measured)",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Supported,
            basis: "cited: user settings live in the config directory; project .claude/settings*.json \
                    and .mcp.json still layer on top",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Conditional,
            basis: "measured: credential environment variables override it; the per-directory \
                    .credentials.json and macOS Keychain entry are cited, not measured",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "cited: history and project state moving with the directory is community-sourced only",
        },
    ],
    env: &[EnvOverride { name: VAR, sensitive: false }],
    conflicts: &[],
};

/// The Claude Code adapter.
#[derive(Debug, Clone, Copy)]
pub struct Claude;

impl Adapter for Claude {
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
