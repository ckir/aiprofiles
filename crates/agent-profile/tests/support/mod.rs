//! Shared helpers for integration tests (SP0 design §3.4, SP1 design §8).
//!
//! Each test file includes this module with `mod support;` and uses only part of it.
#![allow(dead_code)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Every `fake-agent` control variable (SP1 design §8.1).
pub const FIXTURE_VARS: [&str; 7] = [
    "FAKE_AGENT_EXIT",
    "FAKE_AGENT_ECHO_ENV",
    "FAKE_AGENT_STDIN",
    "FAKE_AGENT_STDERR",
    "FAKE_AGENT_SLEEP_MS",
    "FAKE_AGENT_SPAWN_SLEEPER",
    "FAKE_AGENT_CTRL_C_EXIT",
];

/// Wrapper variables a developer's shell could leak into a test run.
pub const WRAPPER_VARS: [&str; 3] =
    ["AGENT_PROFILE_HOME", "FAKE_AGENT_HOME", "AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS"];

/// A `Command` for the `fake-agent` fixture with a clean fixture environment.
///
/// The fixture is a binary of this package, so cargo builds it before any test runs and
/// `CARGO_BIN_EXE_fake-agent` points at it; tests never invoke cargo themselves.
///
/// `Command` inherits the parent's environment, so a control variable exported in the shell running
/// the tests would silently change every fixture run. Tests set the variables they mean to test on top
/// of this baseline.
pub fn fake_agent() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fake-agent"));
    for name in FIXTURE_VARS {
        command.env_remove(name);
    }
    command
}

/// A temporary application root whose `config.toml` points `agents.fake.executable` at the fixture.
pub struct Root {
    pub dir: tempfile::TempDir,
}

impl Root {
    pub fn new() -> Root {
        let root = Root::empty();
        root.write_config(&format!(
            "[agents.fake]\nexecutable = {:?}\n",
            env!("CARGO_BIN_EXE_fake-agent")
        ));
        root
    }

    pub fn empty() -> Root {
        Root { dir: tempfile::tempdir().unwrap() }
    }

    pub fn path(&self) -> &Path {
        self.dir.path()
    }

    pub fn write_config(&self, text: &str) {
        std::fs::write(self.path().join("config.toml"), text).unwrap();
    }

    pub fn profile_dir(&self, profile: &str) -> PathBuf {
        self.path().join("profiles").join(profile).join("fake")
    }

    /// `agent-profile` with this root, a clean fixture and wrapper environment, and the given args.
    pub fn agent_profile<I, S>(&self, args: I) -> Command
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        let mut command = Command::new(env!("CARGO_BIN_EXE_agent-profile"));
        for name in FIXTURE_VARS.iter().chain(WRAPPER_VARS.iter()) {
            command.env_remove(name);
        }
        command.env("AGENT_PROFILE_HOME", self.path());
        command.args(args.into_iter().map(Into::into));
        command
    }
}

/// An argument that is a valid OS string but not valid UTF-8.
#[cfg(unix)]
pub fn non_utf8() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(vec![0x66, 0xff])
}

/// An argument that is a valid OS string but not valid UTF-8: a lone surrogate.
#[cfg(windows)]
pub fn non_utf8() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0x0066, 0xD800])
}

/// Parses the fixture's one-line JSON report.
pub fn report(stdout: &[u8]) -> serde_json::Value {
    serde_json::from_slice(stdout).unwrap_or_else(|e| {
        panic!("not a fake-agent report ({e}): {}", String::from_utf8_lossy(stdout))
    })
}
