//! Profile-name validation (spec §6) and the reserved command words (spec §5.3).

use std::fmt;

/// The v0.1 reserved command words (spec §5.3). A profile name must not equal one, ignoring ASCII case.
pub const RESERVED_WORDS: [&str; 13] = [
    "agents",
    "profiles",
    "status",
    "list",
    "create",
    "delete",
    "current",
    "resolve",
    "doctor",
    "link",
    "unlink",
    "repositories",
    "completions",
];

/// Windows device names (spec §6), rejected with or without an extension.
const WINDOWS_DEVICE_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Whether `word` is a reserved command word, ignoring ASCII case.
pub fn is_reserved_word(word: &str) -> bool {
    RESERVED_WORDS.iter().any(|reserved| reserved.eq_ignore_ascii_case(word))
}

/// Which rule set applies. The Windows-only rules of spec §6 are a parameter so every OS tests both.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Unix,
    Windows,
}

impl Platform {
    /// The platform this binary runs on.
    pub fn host() -> Platform {
        if cfg!(windows) { Platform::Windows } else { Platform::Unix }
    }
}

/// Why a profile name is invalid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidReason {
    Empty,
    BadFirstChar(char),
    BadChar(char),
    Reserved(&'static str),
    WindowsDeviceName,
    WindowsTrailingDot,
}

impl fmt::Display for InvalidReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InvalidReason::Empty => write!(f, "a profile name must not be empty"),
            InvalidReason::BadFirstChar(c) => {
                write!(f, "a profile name must begin with an ASCII letter or digit, not {c:?}")
            }
            InvalidReason::BadChar(c) => write!(
                f,
                "a profile name may contain only ASCII letters, digits, '.', '_' and '-', not {c:?}"
            ),
            InvalidReason::Reserved(word) => {
                write!(f, "\"{word}\" is a reserved command word and cannot be a profile name")
            }
            InvalidReason::WindowsDeviceName => {
                write!(f, "the name is a reserved Windows device name")
            }
            InvalidReason::WindowsTrailingDot => {
                write!(f, "on Windows a profile name must not end with a dot")
            }
        }
    }
}

/// A validated profile name (spec §6).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ProfileName(String);

impl ProfileName {
    /// Validates `name` against every spec §6 rule for `platform`.
    pub fn parse(name: &str, platform: Platform) -> Result<ProfileName, InvalidReason> {
        let mut chars = name.chars();
        let first = chars.next().ok_or(InvalidReason::Empty)?;
        if !first.is_ascii_alphanumeric() {
            return Err(InvalidReason::BadFirstChar(first));
        }
        if let Some(bad) =
            chars.find(|c| !(c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-')))
        {
            return Err(InvalidReason::BadChar(bad));
        }
        if let Some(reserved) = RESERVED_WORDS.iter().find(|word| word.eq_ignore_ascii_case(name)) {
            return Err(InvalidReason::Reserved(reserved));
        }
        if platform == Platform::Windows {
            let stem = name.split('.').next().unwrap_or(name);
            if WINDOWS_DEVICE_NAMES.iter().any(|device| device.eq_ignore_ascii_case(stem)) {
                return Err(InvalidReason::WindowsDeviceName);
            }
            if name.ends_with('.') {
                return Err(InvalidReason::WindowsTrailingDot);
            }
        }
        Ok(ProfileName(name.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ProfileName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// An agent identifier: `[a-z][a-z0-9-]*`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AgentId(String);

impl AgentId {
    /// Returns `None` unless `id` matches `[a-z][a-z0-9-]*`.
    pub fn parse(id: &str) -> Option<AgentId> {
        let mut chars = id.chars();
        let first = chars.next()?;
        let valid = first.is_ascii_lowercase()
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
        valid.then(|| AgentId(id.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for AgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unix(name: &str) -> Result<ProfileName, InvalidReason> {
        ProfileName::parse(name, Platform::Unix)
    }

    fn windows(name: &str) -> Result<ProfileName, InvalidReason> {
        ProfileName::parse(name, Platform::Windows)
    }

    #[test]
    fn valid_names_are_accepted_on_both_platforms() {
        for name in ["work", "Work", "w", "7", "a.b", "a_b", "a-b", "personal-2", "WORK.v1_x-y"] {
            assert_eq!(unix(name).unwrap().as_str(), name);
            assert_eq!(windows(name).unwrap().as_str(), name);
        }
    }

    #[test]
    fn empty_name_is_rejected() {
        assert_eq!(unix(""), Err(InvalidReason::Empty));
        assert_eq!(windows(""), Err(InvalidReason::Empty));
    }

    #[test]
    fn name_must_begin_with_letter_or_digit() {
        for (name, first) in
            [(".a", '.'), ("_a", '_'), ("-a", '-'), (".", '.'), ("..", '.'), (" a", ' ')]
        {
            assert_eq!(unix(name), Err(InvalidReason::BadFirstChar(first)), "{name:?}");
            assert_eq!(windows(name), Err(InvalidReason::BadFirstChar(first)), "{name:?}");
        }
    }

    #[test]
    fn forbidden_characters_are_rejected() {
        for (name, bad) in [
            ("a/b", '/'),
            ("a\\b", '\\'),
            ("a b", ' '),
            ("a\tb", '\t'),
            ("a\0b", '\0'),
            ("a\u{7f}", '\u{7f}'),
            ("a:b", ':'),
            ("a*b", '*'),
            ("caf\u{e9}", '\u{e9}'),
        ] {
            assert_eq!(unix(name), Err(InvalidReason::BadChar(bad)), "{name:?}");
            assert_eq!(windows(name), Err(InvalidReason::BadChar(bad)), "{name:?}");
        }
    }

    #[test]
    fn reserved_words_are_rejected_in_any_case() {
        for word in RESERVED_WORDS {
            let upper = word.to_ascii_uppercase();
            let mixed: String = word
                .chars()
                .enumerate()
                .map(|(i, c)| if i % 2 == 0 { c.to_ascii_uppercase() } else { c })
                .collect();
            for candidate in [word.to_string(), upper, mixed] {
                assert_eq!(unix(&candidate), Err(InvalidReason::Reserved(word)), "{candidate}");
                assert_eq!(windows(&candidate), Err(InvalidReason::Reserved(word)), "{candidate}");
            }
        }
    }

    #[test]
    fn windows_device_names_are_rejected_only_on_windows() {
        let mut names: Vec<String> =
            ["CON", "PRN", "AUX", "NUL"].iter().map(|s| s.to_string()).collect();
        for n in 1..=9 {
            names.push(format!("COM{n}"));
            names.push(format!("LPT{n}"));
        }
        for name in names {
            for candidate in [
                name.clone(),
                name.to_ascii_lowercase(),
                format!("{name}.txt"),
                format!("{name}.a.b"),
            ] {
                assert_eq!(
                    windows(&candidate),
                    Err(InvalidReason::WindowsDeviceName),
                    "{candidate}"
                );
                assert!(unix(&candidate).is_ok(), "{candidate}");
            }
        }
        for not_device in ["CONSOLE", "COM10", "LPT0", "NULL", "AUXX"] {
            assert!(windows(not_device).is_ok(), "{not_device}");
        }
    }

    #[test]
    fn trailing_dot_is_rejected_only_on_windows() {
        assert_eq!(windows("work."), Err(InvalidReason::WindowsTrailingDot));
        assert!(unix("work.").is_ok());
    }

    #[test]
    fn agent_id_syntax() {
        for id in ["fake", "claude", "a", "gemini-cli", "a1-2"] {
            assert_eq!(AgentId::parse(id).unwrap().as_str(), id);
        }
        for id in ["", "Fake", "1a", "-a", "a_b", "a.b", "a b", "\u{e9}"] {
            assert!(AgentId::parse(id).is_none(), "{id:?}");
        }
    }

    #[test]
    fn is_reserved_word_ignores_ascii_case() {
        assert!(is_reserved_word("CREATE"));
        assert!(is_reserved_word("Link"));
        assert!(!is_reserved_word("work"));
    }
}
