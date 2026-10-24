// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

//! Continue CLI: `--config <profile config.yaml>` (SP4 design §8, table row; §9 outcome 5 — the flag is
//! real and accepted, but the measured delta at the watched locations was empty).

use std::path::PathBuf;

use super::{
    Adapter, AdapterEvidence, AdapterMetadata, Capability, CapabilityClaim, CapabilityState,
    ConflictOption, Mechanism, PathKind, PlanContext, PlannedLaunch, SupportLevel, profile_dir,
};
use crate::config::AppRoot;
use crate::error::Result;
use crate::name::ProfileName;

const FLAG: &str = "--config";
const FILE_NAME: &str = "config.yaml";

/// Every spelling of the option Continue launches with; `--config` first, so it is the one reported.
/// Only this spelling was measured — the probe's candidate sweep probed file contents, not flag
/// spellings, so Q6 stays unanswered and no abbreviation may ride along unmeasured (tracked debt).
const CONFIG_OPTION: ConflictOption = ConflictOption { long: &[FLAG], short: None };

/// A zero-byte `config.yaml`: the smallest content the agent accepts (candidate sweep: `""`, `# nothing`
/// and `{}` all exited 0) that sets no option, so it is what ships per §8.4 — a config-file adapter
/// needs the file to exist before `cn` reads it.
pub const INITIAL_CONFIG: &[u8] = b"";

static METADATA: AdapterMetadata = AdapterMetadata {
    id: "continue",
    executable: "cn",
    mechanism: Mechanism::FlagFile(CONFIG_OPTION),
    support: SupportLevel::Experimental,
    evidence: AdapterEvidence {
        mechanism_id: "continue-config-file-v1",
        verified_at: "2026-10-17",
        upstream_version: "1.5.47",
        source_url: "measured",
        notes: "measured in a sandbox at 1.5.47: --help lists `--config <path>  Configuration file for \
                the assistant` and a launch with it exits 0; the delta at both watched locations was \
                `(no change: nothing was created, moved or removed at the watched locations)` — the agent \
                writes nothing before it needs the network (§9 outcome 5); the candidate sweep accepted \
                the empty file, `# nothing` and {} (all exit 0), and the strings scan found CONTINUE_API_KEY, \
                ANTHROPIC_API_KEY and OPENAI_API_KEY documented",
    },
    capabilities: &[
        CapabilityClaim {
            capability: Capability::ConfigIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the probe's delta at both watched locations — the target and the default \
                    `/home/agent/.continue` — was `(no change: nothing was created, moved or removed at the \
                    watched locations)`, so whether the agent's own config writes would follow a configured \
                    file was not observable while the agent writes nothing before it needs the network \
                    (§9 outcome 5)",
        },
        CapabilityClaim {
            capability: Capability::CredentialIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: no login/auth step ran and no credential-bearing file surfaced anywhere in \
                    the delta, so where stored credentials live was not established; the documented bypass \
                    variables are `CONTINUE_API_KEY`, `ANTHROPIC_API_KEY` and `OPENAI_API_KEY` \
                    (strings: `documented CONTINUE_API_KEY: present`)",
        },
        CapabilityClaim {
            capability: Capability::StateIsolation,
            state: CapabilityState::Unknown,
            basis: "unmeasured: the agent's only pre-network write was `.continue/logs/cn.log`, present at \
                    the default location at baseline, and the measured delta recorded \
                    `(no change: nothing was created, moved or removed at the watched locations)`, so no \
                    state write was observable to place (§9 outcome 5)",
        },
    ],
    env: &[],
    // Empty on purpose. `--config` is the mechanism's own option, and `check_conflicts` scans that
    // alongside this list, so naming it here again refused nothing extra. No abbreviated spelling was
    // measured (Q6 unanswered), so there is no list behind it either.
    conflicts: &[],
};

/// The Continue adapter.
#[derive(Debug, Clone, Copy)]
pub struct Continue;

impl Adapter for Continue {
    fn metadata(&self) -> &'static AdapterMetadata {
        &METADATA
    }

    fn paths(&self, root: &AppRoot, profile: &ProfileName) -> Vec<(PathBuf, PathKind)> {
        let dir = profile_dir(root, profile, METADATA.id);
        let file = dir.join(FILE_NAME);
        vec![(dir, PathKind::Dir), (file, PathKind::File { contents: INITIAL_CONFIG })]
    }

    fn plan(&self, ctx: &PlanContext<'_>) -> Result<PlannedLaunch> {
        METADATA.mechanism.plan(self, ctx)
    }
}
