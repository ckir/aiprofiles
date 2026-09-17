//! Declarative adapter metadata: support level, capabilities, evidence, environment declarations, conflicting
//! options and profile presence (spec §3, §8, §21, §22, §28; SP2 design §4.2).

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

/// How far an adapter is proven, separately from its capabilities (spec §3, §37).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportLevel {
    /// Evidence entry, every capability claimed, and the contract suite passes.
    Proven,
    Experimental,
}

/// Where an adapter's mechanism was verified (spec §28, extended with version and source).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterEvidence {
    pub mechanism_id: &'static str,
    /// ISO date, `YYYY-MM-DD`.
    pub verified_at: &'static str,
    pub upstream_version: &'static str,
    /// A URL, or `measured` with the method in `notes`.
    pub source_url: &'static str,
    pub notes: &'static str,
}

/// An isolation property a profile may provide (SP2 design §4.2 definitions).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    /// The profile replaces the user-level configuration the agent reads from its default user location.
    ConfigIsolation,
    /// Stored credentials (files or OS keychain entries) are separated per profile.
    CredentialIsolation,
    /// History, sessions and caches are separated per profile.
    StateIsolation,
}

impl Capability {
    /// Every capability, so each adapter can be checked for claiming all of them.
    pub const ALL: [Capability; 3] =
        [Capability::ConfigIsolation, Capability::CredentialIsolation, Capability::StateIsolation];
}

/// How strongly a capability holds (spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityState {
    Supported,
    NotSupported,
    NotGuaranteed,
    Conditional,
    Unknown,
}

/// One capability claim and the evidence-scoped reason for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapabilityClaim {
    pub capability: Capability,
    pub state: CapabilityState,
    pub basis: &'static str,
}

/// A variable an adapter's plan may set, and whether its value must never be printed (spec §22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnvOverride {
    pub name: &'static str,
    pub sensitive: bool,
}

/// An agent option proven to control the same mechanism the adapter uses (spec §21).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictOption {
    /// Every accepted spelling, each starting with `--`.
    pub long: &'static [&'static str],
    pub short: Option<char>,
}

impl ConflictOption {
    /// The canonical spelling: the first long form. Every `ConflictOption` in a `static AdapterMetadata`
    /// has at least one — `metadata_invariants` walks them all and asserts each starts with `--`, which an
    /// empty list could not satisfy meaningfully.
    ///
    /// This is the *only* place a flag mechanism's spelling comes from, so the option a plan passes, the
    /// sentence the report prints and the token the evidence pipeline reads are one string, not three.
    pub fn spelling(&self) -> &'static str {
        self.long.first().expect("a conflict option declares at least one long spelling")
    }

    /// Whether `arg` selects this option. Works on encoded bytes, so non-UTF-8 arguments are scanned too;
    /// every spelling is ASCII, which makes the byte comparison exact on every platform.
    pub fn matches(&self, arg: &OsStr) -> bool {
        let bytes = arg.as_encoded_bytes();
        let long = self.long.iter().any(|spelling| {
            let spelling = spelling.as_bytes();
            bytes == spelling
                || (bytes.len() > spelling.len()
                    && bytes.starts_with(spelling)
                    && bytes[spelling.len()] == b'=')
        });
        let short = self.short.is_some_and(|letter| {
            let mut flag = [0u8; 4];
            let flag = format!("-{}", letter.encode_utf8(&mut flag));
            bytes.starts_with(flag.as_bytes())
        });
        long || short
    }
}

/// How an adapter points its agent at the profile: the cross product of {variable, flag} x
/// {directory, file}. This is the complete, closed set — it is the same vocabulary the probe harness
/// speaks (`sandbox/probes/common.sh`, `env:` / `envfile:` / `flagdir:` / `flagfile:`), so a fifth
/// variant here would be a mechanism no probe can exercise.
///
/// # Three renderings, one per consumer — do not collapse them
///
/// 1. [`Display`] — the **path-free** sentence, for error messages. An error message cannot name a
///    path, because `check_conflicts` refuses an argument before any plan exists and therefore before
///    any path has been chosen. It prints the placeholder `<dir>` / `<file>` instead.
/// 2. [`Mechanism::sentence_for`] — the **concrete** sentence that becomes
///    [`PlannedLaunch::mechanism`](super::PlannedLaunch::mechanism), for the launch report. A report
///    line should name the exact path it is about to create, so here the placeholder is the real path.
/// 3. [`Mechanism::probe_token`] — the **machine** identifier the evidence pipeline consumes. The
///    human text above may be reworded freely; this token may not.
///
/// That (1) and (2) differ in form for flag mechanisms is deliberate, not drift: they answer different
/// questions at different times. For environment mechanisms they coincide, because there is no path in
/// the sentence to differ over.
///
/// All three are generated from this one value, and so is the launch itself:
/// [`Mechanism::plan`](super::Mechanism::plan) dispatches on the variant and takes the variable name or
/// flag spelling out of the variant's own payload. An adapter passes no name and no flag, so there is no
/// second string for the first to drift from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mechanism {
    /// A variable set to the profile directory.
    Env(&'static str),
    /// A variable set to a file inside the profile directory.
    EnvFile(&'static str),
    /// An option passed the profile directory.
    FlagDir(ConflictOption),
    /// An option passed a file inside the profile directory.
    FlagFile(ConflictOption),
}

impl Mechanism {
    /// The option this mechanism itself occupies, if it is a flag. `check_conflicts` scans it alongside
    /// the adapter's declared extra `conflicts`, so an adapter cannot accept a flag it uses itself.
    pub fn conflict_option(&self) -> Option<&ConflictOption> {
        match self {
            Mechanism::Env(_) | Mechanism::EnvFile(_) => None,
            Mechanism::FlagDir(option) | Mechanism::FlagFile(option) => Some(option),
        }
    }

    /// The report sentence naming `target`, the path this mechanism points at. Environment mechanisms
    /// name the variable and ignore `target`.
    pub fn sentence_for(&self, target: &Path) -> String {
        match self {
            Mechanism::Env(name) | Mechanism::EnvFile(name) => {
                format!("environment variable {name}")
            }
            Mechanism::FlagDir(option) | Mechanism::FlagFile(option) => {
                format!("argument {} {}", option.spelling(), target.display())
            }
        }
    }

    /// The evidence pipeline's identifier, e.g. `env:CODEX_HOME` or `flagfile:--config`.
    pub fn probe_token(&self) -> String {
        match self {
            Mechanism::Env(name) => format!("env:{name}"),
            Mechanism::EnvFile(name) => format!("envfile:{name}"),
            Mechanism::FlagDir(option) => format!("flagdir:{}", option.spelling()),
            Mechanism::FlagFile(option) => format!("flagfile:{}", option.spelling()),
        }
    }
}

impl fmt::Display for Mechanism {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Mechanism::Env(name) | Mechanism::EnvFile(name) => {
                write!(formatter, "environment variable {name}")
            }
            Mechanism::FlagDir(option) => {
                write!(formatter, "argument {} <dir>", option.spelling())
            }
            Mechanism::FlagFile(option) => {
                write!(formatter, "argument {} <file>", option.spelling())
            }
        }
    }
}

/// Whether a profile exists for an adapter (spec §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilePresence {
    /// Nothing has been created for this profile yet.
    Absent,
    /// Every path the adapter declares exists on disk.
    Materialized,
    /// The profile can be identified from the agent's own profile mechanism, without anything of ours
    /// existing on disk.
    ///
    /// **Reserved, not dead.** No adapter constructs this today and the SP3 review flagged it as unused
    /// code, which it is — but it is unclaimed rather than obsolete. Spec §8 defines it normatively for
    /// an agent that manages named profiles itself, where a profile is real because the agent says so
    /// and `agent-profile` may have created no directory at all. Every adapter shipped so far is
    /// directory-based, so the distinction has not yet had a case to express.
    ///
    /// Deleting it was considered in SP4 and rejected: the variant costs one line, while removing it
    /// would make the first native-profile adapter a change to a public enum — and in the meantime
    /// `presence()` would have to report such a profile as `Absent`, which is a false statement about a
    /// profile the agent itself lists.
    Known,
}

/// Everything about an adapter that does not depend on a launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterMetadata {
    pub id: &'static str,
    /// Executable base name, without `.exe`.
    pub executable: &'static str,
    /// How the agent is pointed at the profile. Generates both the report sentence and the path-free
    /// text used in messages, so neither can disagree with what `plan()` does.
    pub mechanism: Mechanism,
    pub support: SupportLevel,
    pub evidence: AdapterEvidence,
    pub capabilities: &'static [CapabilityClaim],
    /// Every variable `plan()` may set, with its sensitivity.
    pub env: &'static [EnvOverride],
    pub conflicts: &'static [ConflictOption],
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;

    const AIDER_CONFIG: ConflictOption =
        ConflictOption { long: &["--config", "--confi", "--conf", "--con"], short: Some('c') };

    #[test]
    fn long_spellings_match_exactly_or_with_an_equals_value() {
        for arg in ["--config", "--config=f", "--confi", "--conf=", "--con=a=b", "-c", "-cf", "-c="]
        {
            assert!(AIDER_CONFIG.matches(OsStr::new(arg)), "{arg}");
        }
        for arg in
            ["--co", "--code-theme", "--configx", "--conf-x", "config", "-", "--", "-x", "--c"]
        {
            assert!(!AIDER_CONFIG.matches(OsStr::new(arg)), "{arg}");
        }
    }

    #[test]
    fn an_option_without_a_short_form_matches_only_long_spellings() {
        let option = ConflictOption { long: &["--fake-profile"], short: None };
        assert!(option.matches(OsStr::new("--fake-profile=x")));
        assert!(!option.matches(OsStr::new("-f")));
    }

    #[cfg(unix)]
    fn non_utf8_value(prefix: &str) -> OsString {
        use std::os::unix::ffi::OsStringExt;
        let mut bytes = prefix.as_bytes().to_vec();
        bytes.push(0xff);
        OsString::from_vec(bytes)
    }

    #[cfg(windows)]
    fn non_utf8_value(prefix: &str) -> OsString {
        use std::os::windows::ffi::OsStringExt;
        let mut wide: Vec<u16> = prefix.encode_utf16().collect();
        wide.push(0xD800);
        OsString::from_wide(&wide)
    }

    #[test]
    fn the_probe_token_names_the_mechanism_kind() {
        // All four variants, none elided: the token vocabulary is a wire contract with
        // `sandbox/probes/common.sh`, whose `case` has exactly these four arms.
        assert_eq!(Mechanism::Env("CODEX_HOME").probe_token(), "env:CODEX_HOME");
        assert_eq!(
            Mechanism::EnvFile("AIDER_CONFIG_FILE").probe_token(),
            "envfile:AIDER_CONFIG_FILE"
        );
        assert_eq!(Mechanism::FlagDir(AIDER_CONFIG).probe_token(), "flagdir:--config");
        assert_eq!(Mechanism::FlagFile(AIDER_CONFIG).probe_token(), "flagfile:--config");
    }

    #[test]
    fn non_utf8_arguments_are_scanned() {
        assert!(AIDER_CONFIG.matches(&non_utf8_value("--config=")));
        assert!(AIDER_CONFIG.matches(&non_utf8_value("-c")));
        assert!(!AIDER_CONFIG.matches(&non_utf8_value("--cod")));
    }
}
