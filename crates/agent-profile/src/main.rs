//! The `agent-profile` binary. SP0 scaffold: only `--version` and `--help` work.

use std::ffi::OsString;
use std::process::ExitCode;

use clap::Parser;

/// Spec §33: exit code for a CLI usage error.
const USAGE_ERROR: u8 = 2;

/// Select and launch profiles for multiple coding agents.
///
/// This is the SP0 scaffold: nothing is implemented yet. See ROADMAP.md.
#[derive(Parser)]
#[command(name = "agent-profile", version)]
struct Cli {
    /// Everything else is accepted and rejected as not yet implemented.
    #[arg(hide = true, trailing_var_arg = true, allow_hyphen_values = true)]
    rest: Vec<OsString>,
}

fn main() -> ExitCode {
    let _cli = Cli::parse();
    eprintln!(
        "agent-profile: not yet implemented (this build is the SP0 scaffold; see ROADMAP.md)"
    );
    ExitCode::from(USAGE_ERROR)
}
