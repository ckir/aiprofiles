// Copyright (c) 2026 Costas Kirgoussios
//
// This software is licensed under the PolyForm Noncommercial License 1.0.0.
// You may obtain a copy of the License at:
// https://polyformproject.org/licenses/noncommercial/1.0.0/

//! Unix launcher: process replacement (spec §23.1).

use std::os::unix::process::CommandExt;

use super::{LaunchOutcome, LaunchPlan};
use crate::error::{Error, Result};

/// Replaces this process with the agent. Returns only when `exec` fails.
pub(super) fn launch(plan: &LaunchPlan) -> Result<LaunchOutcome> {
    let source = plan.command().exec();
    Err(Error::Launch { executable: plan.executable.clone(), source })
}
