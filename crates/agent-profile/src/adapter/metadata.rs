//! Declarative adapter metadata: support level, capabilities, evidence, environment declarations, conflicting
//! options and profile presence (spec §3, §8, §21, §22, §28; SP2 design §4.2).

use std::ffi::OsStr;

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

/// Whether a profile exists for an adapter (spec §8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfilePresence {
    Absent,
    Materialized,
    Known,
}

/// Everything about an adapter that does not depend on a launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdapterMetadata {
    pub id: &'static str,
    /// Executable base name, without `.exe`.
    pub executable: &'static str,
    /// Path-free mechanism text used in messages, e.g. `argument --config <file>`.
    pub mechanism_summary: &'static str,
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
    fn non_utf8_arguments_are_scanned() {
        assert!(AIDER_CONFIG.matches(&non_utf8_value("--config=")));
        assert!(AIDER_CONFIG.matches(&non_utf8_value("-c")));
        assert!(!AIDER_CONFIG.matches(&non_utf8_value("--cod")));
    }
}
