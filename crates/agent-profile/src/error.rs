//! Structured errors and their exit codes (spec §33).

use std::fmt;
use std::io;
use std::path::PathBuf;

use crate::name::InvalidReason;

/// Why an agent's executable was not found (spec §20).
#[derive(Debug)]
pub enum NotInstalledReason {
    NotOnPath,
    ExplicitMissing(PathBuf),
}

/// Every error `agent-profile` reports. `exit_code` maps each variant to spec §33.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{message}")]
    Usage { message: String },

    #[error("`{command}` is not yet implemented")]
    NotYetImplemented { command: String },

    #[error("{}", unknown_agent_message(agent, known, unknown_configured))]
    UnknownAgent { agent: String, known: Vec<String>, unknown_configured: Vec<String> },

    #[error("{}", not_installed_message(agent, reason, unknown_configured))]
    AgentNotInstalled { agent: String, reason: NotInstalledReason, unknown_configured: Vec<String> },

    #[error("invalid profile name {name:?}: {reason}")]
    InvalidProfileName { name: String, reason: InvalidReason },

    #[error("no profile selected for `{agent}`; name one: agent-profile {agent} <profile>")]
    NoProfile { agent: String },

    #[error("{message}")]
    AppRoot { message: String },

    #[error("{}", config_invalid_message(path, key.as_deref(), detail))]
    ConfigInvalid { path: PathBuf, key: Option<String>, detail: String },

    #[error("could not write {}: {source}", path.display())]
    ConfigWrite { path: PathBuf, source: io::Error },

    #[error("profile directory {}: {source}", path.display())]
    ProfileDir { path: PathBuf, source: io::Error },

    #[error(
        "profile `{requested}` differs only in letter case from the existing profile entry `{existing}`; \
         use `{existing}` or choose a different name"
    )]
    ProfileCaseConflict { requested: String, existing: String },

    #[error(
        "{} is a batch file; agent-profile launches agents directly and never through cmd.exe",
        path.display()
    )]
    UnsupportedExecutable { path: PathBuf },

    #[error("could not launch {}: {source}", executable.display())]
    Launch { executable: PathBuf, source: io::Error },

    #[error("{context}: {source}")]
    Io { context: String, source: io::Error },
}

impl Error {
    /// The spec §33 exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage { .. } | Error::NotYetImplemented { .. } | Error::UnknownAgent { .. } => 2,
            Error::AgentNotInstalled { .. } => 3,
            Error::InvalidProfileName { .. }
            | Error::NoProfile { .. }
            | Error::AppRoot { .. }
            | Error::ConfigInvalid { .. }
            | Error::ConfigWrite { .. }
            | Error::ProfileDir { .. }
            | Error::ProfileCaseConflict { .. } => 4,
            Error::UnsupportedExecutable { .. } | Error::Launch { .. } => 6,
            Error::Io { .. } => 1,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

fn unknown_configured_suffix(unknown_configured: &[String]) -> String {
    if unknown_configured.is_empty() {
        String::new()
    } else {
        format!("; config.toml also configures unknown agents: {}", backticked(unknown_configured))
    }
}

fn backticked(items: &[String]) -> String {
    items.iter().map(|item| format!("`{item}`")).collect::<Vec<_>>().join(", ")
}

fn unknown_agent_message(agent: &str, known: &[String], unknown_configured: &[String]) -> String {
    let known = if known.is_empty() {
        "no agents are available in this build".to_owned()
    } else {
        format!("known agents: {}", backticked(known))
    };
    format!("unknown agent `{agent}` ({known}){}", unknown_configured_suffix(unknown_configured))
}

fn not_installed_message(
    agent: &str,
    reason: &NotInstalledReason,
    unknown_configured: &[String],
) -> String {
    let what = match reason {
        NotInstalledReason::NotOnPath => format!("`{agent}` is not installed: not found in PATH"),
        NotInstalledReason::ExplicitMissing(path) => format!(
            "`{agent}` is not installed: the configured executable {} does not exist or is not a file",
            path.display()
        ),
    };
    format!("{what}{}", unknown_configured_suffix(unknown_configured))
}

fn config_invalid_message(path: &std::path::Path, key: Option<&str>, detail: &str) -> String {
    let key = key.map(|key| format!(" (key `{key}`)")).unwrap_or_default();
    format!(
        "invalid configuration {}{key}: {detail}. agent-profile never rewrites an invalid \
         configuration; fix or move the file.",
        path.display()
    )
}

impl fmt::Display for NotInstalledReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotInstalledReason::NotOnPath => write!(f, "not found in PATH"),
            NotInstalledReason::ExplicitMissing(path) => {
                write!(f, "configured executable {} is missing", path.display())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn io() -> io::Error {
        io::Error::other("x")
    }

    #[test]
    fn exit_codes_follow_spec_33() {
        let cases: Vec<(Error, i32)> = vec![
            (Error::Usage { message: "m".into() }, 2),
            (Error::NotYetImplemented { command: "doctor".into() }, 2),
            (
                Error::UnknownAgent {
                    agent: "a".into(),
                    known: vec![],
                    unknown_configured: vec![],
                },
                2,
            ),
            (
                Error::AgentNotInstalled {
                    agent: "a".into(),
                    reason: NotInstalledReason::NotOnPath,
                    unknown_configured: vec![],
                },
                3,
            ),
            (Error::InvalidProfileName { name: "".into(), reason: InvalidReason::Empty }, 4),
            (Error::NoProfile { agent: "a".into() }, 4),
            (Error::AppRoot { message: "m".into() }, 4),
            (Error::ConfigInvalid { path: "c".into(), key: None, detail: "d".into() }, 4),
            (Error::ConfigWrite { path: "c".into(), source: io() }, 4),
            (Error::ProfileDir { path: "p".into(), source: io() }, 4),
            (Error::ProfileCaseConflict { requested: "WORK".into(), existing: "work".into() }, 4),
            (Error::UnsupportedExecutable { path: "a.cmd".into() }, 6),
            (Error::Launch { executable: "a".into(), source: io() }, 6),
            (Error::Io { context: "c".into(), source: io() }, 1),
        ];
        for (error, code) in cases {
            assert_eq!(error.exit_code(), code, "{error:?}");
        }
    }

    #[test]
    fn unknown_agent_message_lists_known_and_unknown_configured() {
        let none =
            Error::UnknownAgent { agent: "zzz".into(), known: vec![], unknown_configured: vec![] };
        assert_eq!(none.to_string(), "unknown agent `zzz` (no agents are available in this build)");
        let some = Error::UnknownAgent {
            agent: "fakr".into(),
            known: vec!["fake".into()],
            unknown_configured: vec!["fakr".into()],
        };
        assert_eq!(
            some.to_string(),
            "unknown agent `fakr` (known agents: `fake`); config.toml also configures unknown agents: `fakr`"
        );
    }

    #[test]
    fn config_invalid_message_names_file_key_and_guidance() {
        let error = Error::ConfigInvalid {
            path: "/r/config.toml".into(),
            key: Some("agents.fake.path".into()),
            detail: "unknown field".into(),
        };
        let message = error.to_string();
        assert!(message.contains("/r/config.toml"), "{message}");
        assert!(message.contains("`agents.fake.path`"), "{message}");
        assert!(message.contains("never rewrites an invalid configuration"), "{message}");
    }
}
