//! Structured errors and their exit codes (spec §33).

use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use crate::name::InvalidReason;

/// Why an agent's executable was not found (spec §20).
#[derive(Debug)]
pub enum NotInstalledReason {
    /// No absolute `PATH` entry holds it; `ignored_relative` relative entries were skipped.
    NotOnPath {
        ignored_relative: usize,
    },
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

    #[error(
        "`{option}` conflicts with how agent-profile selects the {agent} profile ({mechanism}); remove it from \
         the agent arguments"
    )]
    ArgumentConflict { agent: String, option: String, mechanism: &'static str },

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

    #[error("profile path {}: {source}", path.display())]
    ProfileDir { path: PathBuf, source: io::Error },

    #[error(
        "profile `{requested}` differs only in letter case from the existing profile entry `{existing}`; \
         use `{existing}` or choose a different name"
    )]
    ProfileCaseConflict { requested: String, existing: String },

    #[error("{}", unsupported_message(agent, path, config_file))]
    UnsupportedExecutable { agent: String, path: PathBuf, config_file: PathBuf },

    #[error("could not launch {}: {source}", executable.display())]
    Launch { executable: PathBuf, source: io::Error },

    #[error("{context}: {source}")]
    Io { context: String, source: io::Error },
}

impl Error {
    /// The spec §33 exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            Error::Usage { .. }
            | Error::NotYetImplemented { .. }
            | Error::UnknownAgent { .. }
            | Error::ArgumentConflict { .. } => 2,
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
    format!("`{agent}` is not installed: {reason}{}", unknown_configured_suffix(unknown_configured))
}

fn unsupported_message(agent: &str, path: &Path, config_file: &Path) -> String {
    format!(
        "`{agent}` resolves to {}, which agent-profile cannot launch without a shell. Install the agent's native \
         executable (for example the vendor's standalone installer) or set [agents.{agent}] executable = \
         \"<absolute path to a native .exe>\" in {}",
        path.display(),
        config_file.display()
    )
}

fn config_invalid_message(path: &Path, key: Option<&str>, detail: &str) -> String {
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
            NotInstalledReason::NotOnPath { ignored_relative: 0 } => write!(f, "not found in PATH"),
            NotInstalledReason::NotOnPath { ignored_relative } => {
                write!(
                    f,
                    "not found in PATH ({ignored_relative} relative PATH entries were ignored)"
                )
            }
            NotInstalledReason::ExplicitMissing(path) => write!(
                f,
                "the configured executable {} does not exist or is not a file",
                path.display()
            ),
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
                Error::ArgumentConflict {
                    agent: "aider".into(),
                    option: "--conf".into(),
                    mechanism: "argument --config <file>",
                },
                2,
            ),
            (
                Error::AgentNotInstalled {
                    agent: "a".into(),
                    reason: NotInstalledReason::NotOnPath { ignored_relative: 0 },
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
            (
                Error::UnsupportedExecutable {
                    agent: "a".into(),
                    path: "a.cmd".into(),
                    config_file: "c".into(),
                },
                6,
            ),
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

    #[test]
    fn not_installed_messages_share_one_text_and_count_ignored_entries() {
        let error = |reason| Error::AgentNotInstalled {
            agent: "codex".into(),
            reason,
            unknown_configured: vec![],
        };
        assert_eq!(
            error(NotInstalledReason::NotOnPath { ignored_relative: 0 }).to_string(),
            "`codex` is not installed: not found in PATH"
        );
        assert_eq!(
            error(NotInstalledReason::NotOnPath { ignored_relative: 2 }).to_string(),
            "`codex` is not installed: not found in PATH (2 relative PATH entries were ignored)"
        );
        let missing = NotInstalledReason::ExplicitMissing("/x/codex".into());
        assert_eq!(
            missing.to_string(),
            format!(
                "the configured executable {} does not exist or is not a file",
                Path::new("/x/codex").display()
            )
        );
    }

    #[test]
    fn conflict_and_unsupported_messages() {
        let conflict = Error::ArgumentConflict {
            agent: "aider".into(),
            option: "--conf".into(),
            mechanism: "argument --config <file>",
        };
        assert_eq!(
            conflict.to_string(),
            "`--conf` conflicts with how agent-profile selects the aider profile (argument --config <file>); \
             remove it from the agent arguments"
        );
        let unsupported = Error::UnsupportedExecutable {
            agent: "codex".into(),
            path: "codex.cmd".into(),
            config_file: "config.toml".into(),
        };
        assert_eq!(
            unsupported.to_string(),
            "`codex` resolves to codex.cmd, which agent-profile cannot launch without a shell. Install the \
             agent's native executable (for example the vendor's standalone installer) or set \
             [agents.codex] executable = \"<absolute path to a native .exe>\" in config.toml"
        );
    }

    #[test]
    fn profile_path_message_fits_files_and_directories() {
        let error = Error::ProfileDir {
            path: ".aider.conf.yml".into(),
            source: io::Error::other("denied"),
        };
        assert_eq!(error.to_string(), "profile path .aider.conf.yml: denied");
    }
}
