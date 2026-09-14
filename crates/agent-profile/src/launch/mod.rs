//! `LaunchPlan` (spec §4), environment overrides (spec §22), launch semantics (spec §23, §24)
//! and the launcher API (spec §25).

use std::ffi::OsString;
use std::path::PathBuf;
use std::process::Command;

use crate::error::Result;

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

/// What an adapter asks the launcher to run (spec §4). `env` is an override set, not a full environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchPlan {
    pub executable: PathBuf,
    pub args: Vec<OsString>,
    pub env: Vec<(OsString, OsString)>,
    pub cwd: Option<PathBuf>,
}

impl LaunchPlan {
    /// The single conversion both launchers use: the inherited environment plus the overrides, the
    /// arguments verbatim, the working directory only when set, and inherited stdio.
    pub fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.args(&self.args);
        for (key, value) in &self.env {
            command.env(key, value);
        }
        if let Some(cwd) = &self.cwd {
            command.current_dir(cwd);
        }
        command
    }
}

/// How a launch ended (spec §25).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchOutcome {
    /// Unix: `exec` never returns on success, so this is never constructed there. Kept for spec §25.
    ReplacedProcess,
    /// Windows: the child's full 32-bit exit code.
    Exited(i32),
}

/// Launches the plan: `exec` on Unix, a waited child on Windows. `verbose` enables diagnostics on stderr.
pub fn launch(plan: &LaunchPlan, verbose: bool) -> Result<LaunchOutcome> {
    #[cfg(unix)]
    {
        let _ = verbose;
        unix::launch(plan)
    }
    #[cfg(windows)]
    {
        windows::launch(plan, verbose)
    }
}
