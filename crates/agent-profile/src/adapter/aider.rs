//! Aider: `--config <profile file>` (SP2 design §5.3).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    ConflictOption, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel,
    config_file_arg_plan, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const FLAG: &str = "--config";
const FILE_NAME: &str = ".aider.conf.yml";

/// Every spelling of the option Aider launches with; `--config` first, so it is the one reported.
const CONFIG_OPTION: ConflictOption =
    ConflictOption { long: &[FLAG, "--confi", "--conf", "--con"], short: Some('c') };

/// An empty YAML mapping: Aider rejects an empty or comment-only file, and `{}` sets no option (design D5).
pub const INITIAL_CONFIG: &[u8] = b"{}\n";

/// Shown on every launch.
pub const LAYERING_NOTE: &str =
    "--config is layered over .aider.conf.yml in the working directory, git root and home";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "aider",
    executable: "aider",
    mechanism: Mechanism::FlagFile(CONFIG_OPTION),
    support: SupportLevel::Proven,
    evidence: AdapterEvidence {
        mechanism_id: "aider-config-file-v1",
        verified_at: "2026-09-15",
        upstream_version: "0.86.2",
        source_url: "measured",
        notes: "measured in a sandbox: --config layers over default .aider.conf.yml files; -c, --con, --conf and \
                --confi select the config file; a missing, empty or comment-only file exits 2; {} is accepted",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::NotGuaranteed,
            basis: "measured: the user-level ~/.aider.conf.yml, repository and cwd config files, .env \
                    files and AIDER_* variables still apply",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: API keys come from the environment, .env files and config files",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::NotSupported,
            basis: "measured: history files are written in the working directory",
        },
    ],
    env: &[],
    // Redundant with the mechanism's own option, which `check_conflicts` now scans too. Kept so this
    // commit is a pure refactor; a follow-up can prune it to `&[]`.
    conflicts: &[CONFIG_OPTION],
};

/// The Aider adapter.
#[derive(Debug, Clone, Copy)]
pub struct Aider;

impl Adapter for Aider {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        let dir = profile_dir(root, profile, METADATA.id);
        let file = dir.join(FILE_NAME);
        vec![(dir, PathKind::Dir), (file, PathKind::File { contents: INITIAL_CONFIG })]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        let mut planned = config_file_arg_plan(self, ctx, FLAG)?;
        planned.notes.push(LAYERING_NOTE.to_owned());
        Ok(planned)
    }
}
