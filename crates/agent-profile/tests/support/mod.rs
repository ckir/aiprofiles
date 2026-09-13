//! Shared helpers for integration tests (SP0 design §3.4).

use std::process::Command;

/// A `Command` for the `fake-agent` fixture with a clean fixture environment.
///
/// The fixture is a binary of this package, so cargo builds it before any test runs and
/// `CARGO_BIN_EXE_fake-agent` points at it; tests never invoke cargo themselves.
///
/// `Command` inherits the parent's environment, so a `FAKE_AGENT_EXIT` or `FAKE_AGENT_ECHO_ENV`
/// exported in the shell running the tests would silently change every fixture run. Tests set the
/// variables they mean to test on top of this baseline.
pub fn fake_agent() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fake-agent"));
    command.env_remove("FAKE_AGENT_EXIT").env_remove("FAKE_AGENT_ECHO_ENV");
    command
}
