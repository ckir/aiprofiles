//! Test-only stand-in for a coding agent (SP0 design §3.3, SP1 design §8.1).
//!
//! Prints one JSON object `{"argv": [...], "cwd": "...", "env": {...}, "pid": N}` to stdout and exits with
//! the code in `FAKE_AGENT_EXIT` (default 0). `env` holds only the variables named in the
//! comma-separated `FAKE_AGENT_ECHO_ENV`. Any fixture error — a non-UTF-8 argument, cwd or echoed
//! value, or an invalid control variable — prints nothing to stdout and exits 125.
//!
//! Control variables added in SP1:
//! - `FAKE_AGENT_STDIN=1`: read all of stdin first and report it as `"stdin"`.
//! - `FAKE_AGENT_STDERR=<text>`: write `<text>` to stderr after the report.
//! - `FAKE_AGENT_SLEEP_MS=<u64>`: after the report, sleep that long before exiting.
//! - `FAKE_AGENT_SPAWN_SLEEPER=<u64>`: spawn a detached copy of itself that only sleeps that long, and
//!   report its PID as `"sleeper_pid"`.
//! - `FAKE_AGENT_CTRL_C_EXIT=<u8>` (Windows only): on Ctrl-C or Ctrl-Break, sleep 300 ms, then exit with
//!   `<u8>`. The handler is installed before the report is printed.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::{Read, Write};
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

/// Reserved for fixture errors, so a test can tell them apart from a requested exit code.
const FIXTURE_ERROR: u8 = 125;

/// Every control variable, so a spawned sleeper can be given a clean environment.
const CONTROL_VARS: [&str; 7] = [
    "FAKE_AGENT_EXIT",
    "FAKE_AGENT_ECHO_ENV",
    "FAKE_AGENT_STDIN",
    "FAKE_AGENT_STDERR",
    "FAKE_AGENT_SLEEP_MS",
    "FAKE_AGENT_SPAWN_SLEEPER",
    "FAKE_AGENT_CTRL_C_EXIT",
];

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
    let code = parsed::<u8>("FAKE_AGENT_EXIT")?.unwrap_or(0);
    let sleep_ms = parsed::<u64>("FAKE_AGENT_SLEEP_MS")?;
    let sleeper_ms = parsed::<u64>("FAKE_AGENT_SPAWN_SLEEPER")?;
    let ctrl_c_exit = parsed::<u8>("FAKE_AGENT_CTRL_C_EXIT")?;
    let read_stdin = match std::env::var_os("FAKE_AGENT_STDIN") {
        None => false,
        Some(value) if value == "1" => true,
        Some(value) => return Err(format!("FAKE_AGENT_STDIN must be 1, got {value:?}")),
    };
    let stderr_text =
        std::env::var_os("FAKE_AGENT_STDERR").map(|v| utf8(v, "FAKE_AGENT_STDERR")).transpose()?;

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

    if let Some(exit) = ctrl_c_exit {
        install_ctrl_c_exit(exit)?;
    }

    let stdin = if read_stdin {
        let mut bytes = Vec::new();
        std::io::stdin().read_to_end(&mut bytes).map_err(|e| format!("cannot read stdin: {e}"))?;
        Some(String::from_utf8(bytes).map_err(|_| "stdin is not valid UTF-8".to_owned())?)
    } else {
        None
    };

    let mut report = serde_json::json!({
        "argv": argv,
        "cwd": cwd,
        "env": env,
        "pid": std::process::id(),
    });
    if let Some(stdin) = stdin {
        report["stdin"] = stdin.into();
    }
    if let Some(ms) = sleeper_ms {
        report["sleeper_pid"] = spawn_sleeper(ms)?.into();
    }

    let mut stdout = std::io::stdout();
    writeln!(stdout, "{report}")
        .and_then(|()| stdout.flush())
        .map_err(|e| format!("stdout: {e}"))?;
    if let Some(text) = stderr_text {
        let mut stderr = std::io::stderr();
        let _ = stderr.write_all(text.as_bytes());
        let _ = stderr.flush();
    }
    if let Some(ms) = sleep_ms {
        std::thread::sleep(Duration::from_millis(ms));
    }
    Ok(code)
}

/// Parses an optional control variable. Unset means `None`; anything unparsable is a fixture error.
fn parsed<T: std::str::FromStr>(name: &str) -> Result<Option<T>, String> {
    match std::env::var_os(name) {
        None => Ok(None),
        Some(value) => value
            .to_str()
            .and_then(|s| s.parse::<T>().ok())
            .map(Some)
            .ok_or_else(|| format!("{name} has an invalid value {value:?}")),
    }
}

fn utf8(value: OsString, what: &str) -> Result<String, String> {
    value.into_string().map_err(|raw| format!("{what} is not valid UTF-8: {raw:?}"))
}

fn spawn_sleeper(ms: u64) -> Result<u32, String> {
    let exe = std::env::current_exe().map_err(|e| format!("cannot locate fake-agent: {e}"))?;
    let child =
        sleeper_command(exe, ms).spawn().map_err(|e| format!("cannot spawn the sleeper: {e}"))?;
    Ok(child.id())
}

/// The sleeper gets no control variable except its sleep, so it never spawns another sleeper.
fn sleeper_command(exe: std::path::PathBuf, ms: u64) -> Command {
    let mut command = Command::new(exe);
    for name in CONTROL_VARS {
        command.env_remove(name);
    }
    command
        .env("FAKE_AGENT_SLEEP_MS", ms.to_string())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

#[cfg(windows)]
fn install_ctrl_c_exit(exit: u8) -> Result<(), String> {
    use std::sync::atomic::{AtomicU8, Ordering};
    use windows_sys::Win32::Foundation::TRUE;
    use windows_sys::Win32::System::Console::{
        CTRL_BREAK_EVENT, CTRL_C_EVENT, SetConsoleCtrlHandler,
    };
    use windows_sys::core::BOOL;

    static EXIT: AtomicU8 = AtomicU8::new(0);

    unsafe extern "system" fn handler(event: u32) -> BOOL {
        if event == CTRL_C_EVENT || event == CTRL_BREAK_EVENT {
            std::thread::sleep(Duration::from_millis(300));
            std::process::exit(i32::from(EXIT.load(Ordering::SeqCst)));
        }
        0
    }

    EXIT.store(exit, Ordering::SeqCst);
    // SAFETY: `handler` is a valid `extern "system"` routine for the life of the process.
    if unsafe { SetConsoleCtrlHandler(Some(handler), TRUE) } == 0 {
        return Err(format!(
            "cannot install the console handler: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

#[cfg(not(windows))]
fn install_ctrl_c_exit(_exit: u8) -> Result<(), String> {
    Err("FAKE_AGENT_CTRL_C_EXIT is supported only on Windows".to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn sleeper_environment_removes_every_control_variable_except_its_sleep() {
        let command = sleeper_command("fake-agent".into(), 250);
        let envs: BTreeMap<&OsStr, Option<&OsStr>> = command.get_envs().collect();
        for name in CONTROL_VARS {
            let expected =
                if name == "FAKE_AGENT_SLEEP_MS" { Some(OsStr::new("250")) } else { None };
            assert_eq!(envs.get(OsStr::new(name)), Some(&expected), "{name}");
        }
    }
}
