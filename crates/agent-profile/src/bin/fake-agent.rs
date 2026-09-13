//! Test-only stand-in for a coding agent (SP0 design §3.3).
//!
//! Prints one JSON object `{"argv": [...], "cwd": "...", "env": {...}}` to stdout and exits with
//! the code in `FAKE_AGENT_EXIT` (default 0). `env` holds only the variables named in the
//! comma-separated `FAKE_AGENT_ECHO_ENV`. Any fixture error — a non-UTF-8 argument, cwd or echoed
//! value, or an invalid `FAKE_AGENT_EXIT` — prints nothing to stdout and exits 125.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::process::ExitCode;

/// Reserved for fixture errors, so a test can tell them apart from a requested exit code.
const FIXTURE_ERROR: u8 = 125;

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(message) => {
            eprintln!("fake-agent: {message}");
            ExitCode::from(FIXTURE_ERROR)
        }
    }
}

/// Builds the whole report before printing, so an error never leaves partial output.
fn run() -> Result<u8, String> {
    let code = requested_exit_code()?;

    let argv = std::env::args_os()
        .skip(1)
        .map(|arg| utf8(arg, "argument"))
        .collect::<Result<Vec<_>, _>>()?;

    let cwd =
        std::env::current_dir().map_err(|e| format!("cannot read the current directory: {e}"))?;
    let cwd = utf8(cwd.into_os_string(), "current directory")?;

    let mut env = BTreeMap::new();
    if let Some(list) = std::env::var_os("FAKE_AGENT_ECHO_ENV") {
        let list = utf8(list, "FAKE_AGENT_ECHO_ENV")?;
        for name in list.split(',').filter(|name| !name.is_empty()) {
            if let Some(value) = std::env::var_os(name) {
                env.insert(name.to_owned(), utf8(value, name)?);
            }
        }
    }

    let report = serde_json::json!({ "argv": argv, "cwd": cwd, "env": env });
    println!("{report}");
    Ok(code)
}

/// `FAKE_AGENT_EXIT`: unset means 0; otherwise an integer in `0..=255`.
fn requested_exit_code() -> Result<u8, String> {
    match std::env::var_os("FAKE_AGENT_EXIT") {
        None => Ok(0),
        Some(value) => value
            .to_str()
            .and_then(|s| s.parse::<u8>().ok())
            .ok_or_else(|| format!("FAKE_AGENT_EXIT must be an integer in 0..=255, got {value:?}")),
    }
}

fn utf8(value: OsString, what: &str) -> Result<String, String> {
    value.into_string().map_err(|raw| format!("{what} is not valid UTF-8: {raw:?}"))
}
