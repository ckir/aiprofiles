//! The shell completion model. The model exists only for `clap_complete::generate`: it never parses an
//! argument line, and it mirrors the grammar table on `CommandWord` (scope, flags, positionals), so the
//! generated scripts cannot drift from the hand dispatcher.
//!
//! The bash script needs a workaround. clap_complete 4.6.11 (`aot/shells/bash.rs`) derives the function
//! name from `bin_name.replace('-', "__")` but the `case` labels from `replace('-', CMD_SEP)`, so for a
//! binary whose name contains a hyphen the state labels name a function path that is never reached.
//! `script` rewrites the label back. Remove once upstream names them consistently.

// Removed in Task 6 once `completions` calls these.
#![allow(dead_code)]

use clap::{Arg, ArgAction, Command, ValueHint};
use clap_complete::Shell;

use super::{AgentScope, CommandWord, Positional};
use crate::adapter;

pub(super) const SHELL_NAMES: [&str; 5] = ["bash", "elvish", "fish", "powershell", "zsh"];

/// The exact lower-case spelling only.
pub(super) fn parse_shell(word: &str) -> Option<Shell> {
    match word {
        "bash" => Some(Shell::Bash),
        "elvish" => Some(Shell::Elvish),
        "fish" => Some(Shell::Fish),
        "powershell" => Some(Shell::PowerShell),
        "zsh" => Some(Shell::Zsh),
        _ => None,
    }
}

/// The completion model; callers build it.
pub(super) fn model() -> Command {
    let mut root = Command::new("agent-profile")
        .version(env!("CARGO_PKG_VERSION"))
        .disable_help_subcommand(true);
    for word in CommandWord::ALL {
        if word.scope() != AgentScope::AgentRequired {
            root = root.subcommand(command_model(word, false));
        }
    }
    for id in adapter::known_agents() {
        let mut agent = Command::new(id)
            .disable_help_subcommand(true)
            .arg(flag("dry-run"))
            .arg(flag("verbose"));
        for word in CommandWord::ALL {
            if word.scope() != AgentScope::TopLevelOnly {
                agent = agent.subcommand(command_model(word, true));
            }
        }
        root = root.subcommand(agent);
    }
    root
}

fn command_model(word: CommandWord, with_agent: bool) -> Command {
    let mut command = Command::new(word.as_str()).disable_help_subcommand(true);
    if word.accepts_repo() {
        command = command
            .arg(Arg::new("repo").long("repo").value_name("path").value_hint(ValueHint::DirPath));
    }
    if word.accepts_json() {
        command = command.arg(flag("json"));
    }
    if word.accepts_yes() {
        command = command.arg(flag("yes"));
    }
    match word.positional(with_agent) {
        Positional::RequiredProfile => {
            command = command.arg(Arg::new("profile").value_name("profile").required(true));
        }
        Positional::OptionalProfile => {
            command = command.arg(Arg::new("profile").value_name("profile"));
        }
        Positional::Shell => {
            command = command.arg(
                Arg::new("shell").value_name("shell").required(true).value_parser(SHELL_NAMES),
            );
        }
        Positional::None => {}
    }
    command
}

fn flag(name: &'static str) -> Arg {
    Arg::new(name).long(name).action(ArgAction::SetTrue)
}

/// The completion script for `shell`, as the text to print.
pub(super) fn script(shell: Shell) -> String {
    let mut model = model();
    let mut buffer = Vec::new();
    clap_complete::generate(shell, &mut model, "agent-profile", &mut buffer);
    let text = String::from_utf8_lossy(&buffer).into_owned();
    if shell == Shell::Bash {
        text.replace("agent__subcmd__profile", "agent__profile")
    } else {
        text
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;
    #[cfg(target_os = "linux")]
    use std::path::Path;

    use super::*;

    fn set<'a>(items: impl IntoIterator<Item = &'a str>) -> BTreeSet<String> {
        items.into_iter().map(str::to_owned).collect()
    }

    fn built() -> Command {
        let mut root = model();
        root.build();
        root
    }

    fn arg_ids(command: &Command) -> BTreeSet<String> {
        command.get_arguments().map(|a| a.get_id().to_string()).collect()
    }

    fn sub_names(command: &Command) -> BTreeSet<String> {
        command.get_subcommands().map(|c| c.get_name().to_owned()).collect()
    }

    #[test]
    fn root_subcommands_are_the_top_level_words_and_the_agents() {
        let root = built();
        let mut want = set(CommandWord::ALL
            .iter()
            .filter(|w| w.scope() != AgentScope::AgentRequired)
            .map(|w| w.as_str()));
        want.extend(set(adapter::known_agents()));
        assert_eq!(sub_names(&root), want);
    }

    #[test]
    fn root_arguments_are_help_and_version() {
        assert_eq!(arg_ids(&built()), set(["help", "version"]));
    }

    #[test]
    fn each_agent_offers_the_agent_words_and_the_launch_flags() {
        let root = built();
        let want_words = set(CommandWord::ALL
            .iter()
            .filter(|w| w.scope() != AgentScope::TopLevelOnly)
            .map(|w| w.as_str()));
        for id in adapter::known_agents() {
            let agent = root.find_subcommand(id).expect("agent subcommand");
            assert_eq!(sub_names(agent), want_words, "{id}");
            assert_eq!(arg_ids(agent), set(["dry-run", "verbose", "help"]), "{id}");
        }
    }

    #[test]
    fn each_command_mirrors_the_grammar_table() {
        let root = built();
        let mut checked = 0;
        let mut check = |command: &Command, word: CommandWord, with_agent: bool| {
            let mut want = set(["help"]);
            if word.accepts_repo() {
                want.insert("repo".to_owned());
                let repo = command.get_arguments().find(|a| a.get_id() == "repo").unwrap();
                assert_eq!(repo.get_value_hint(), ValueHint::DirPath);
            }
            if word.accepts_json() {
                want.insert("json".to_owned());
            }
            if word.accepts_yes() {
                want.insert("yes".to_owned());
            }
            match word.positional(with_agent) {
                Positional::RequiredProfile | Positional::OptionalProfile => {
                    want.insert("profile".to_owned());
                }
                Positional::Shell => {
                    want.insert("shell".to_owned());
                }
                Positional::None => {}
            }
            assert_eq!(arg_ids(command), want, "{}", word.as_str());
            assert!(sub_names(command).is_empty(), "{}", word.as_str());
            checked += 1;
        };
        for word in CommandWord::ALL {
            if let Some(command) = root.find_subcommand(word.as_str()) {
                check(command, word, false);
            }
        }
        for id in adapter::known_agents() {
            let agent = root.find_subcommand(id).unwrap();
            for word in CommandWord::ALL {
                if let Some(command) = agent.find_subcommand(word.as_str()) {
                    check(command, word, true);
                }
            }
        }
        assert!(checked > 0);
    }

    fn walk(command: &Command, found_help: &mut bool) {
        for sub in command.get_subcommands() {
            if sub.get_name() == "help" {
                *found_help = true;
            }
            walk(sub, found_help);
        }
    }

    #[test]
    fn no_help_subcommand_anywhere() {
        let mut found = false;
        walk(&built(), &mut found);
        assert!(!found);
    }

    #[test]
    fn script_for_each_shell_names_every_agent_and_word() {
        for shell in [Shell::Bash, Shell::Elvish, Shell::Fish, Shell::PowerShell, Shell::Zsh] {
            let text = script(shell);
            assert!(!text.is_empty(), "{shell}");
            for id in adapter::known_agents() {
                assert!(text.contains(id), "{shell}: {id}");
            }
            for word in CommandWord::ALL {
                assert!(text.contains(word.as_str()), "{shell}: {}", word.as_str());
            }
        }
    }

    #[test]
    fn parse_shell_is_exact_lower_case() {
        let want = [Shell::Bash, Shell::Elvish, Shell::Fish, Shell::PowerShell, Shell::Zsh];
        for (name, shell) in SHELL_NAMES.iter().zip(want) {
            assert_eq!(parse_shell(name), Some(shell));
        }
        for bad in ["Bash", "BASH", "", "nope"] {
            assert_eq!(parse_shell(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn bash_script_has_no_mismatched_state_label() {
        let text = script(Shell::Bash);
        assert!(!text.contains("agent__subcmd__profile"));
        assert!(text.contains("agent__profile__subcmd__claude)"));
    }

    #[cfg(target_os = "linux")]
    fn complete(script: &Path, words: &[&str]) -> BTreeSet<String> {
        let out = std::process::Command::new("bash")
            .args(["--norc", "--noprofile", "-c"])
            .arg(
                r#"source "$1"; shift; COMP_WORDS=("$@"); COMP_CWORD=$((${#COMP_WORDS[@]} - 1)); COMP_LINE="$*"; COMP_POINT=${#COMP_LINE}; _agent__profile agent-profile "${COMP_WORDS[COMP_CWORD]}" "${COMP_WORDS[COMP_CWORD - 1]}"; printf "%s\n" "${COMPREPLY[@]}""#,
            )
            .arg("bash")
            .arg(script)
            .args(words)
            .output()
            .unwrap();
        String::from_utf8_lossy(&out.stdout).lines().map(str::to_owned).collect()
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn bash_script_completes_the_second_and_third_word() {
        use std::io::Write;
        let mut file = tempfile::NamedTempFile::new().unwrap();
        file.write_all(script(Shell::Bash).as_bytes()).unwrap();
        file.flush().unwrap();
        let path = file.path();

        let first = complete(path, &["agent-profile", ""]);
        assert!(first.contains("claude") && first.contains("agents"), "{first:?}");

        let second = complete(path, &["agent-profile", "claude", ""]);
        assert!(second.contains("create") && second.contains("--dry-run"), "{second:?}");
        assert!(!second.contains("agents"), "{second:?}");

        let third = complete(path, &["agent-profile", "claude", "create", ""]);
        assert!(third.contains("--help"), "{third:?}");
        assert!(!third.contains("--json") && !third.contains("--repo"), "{third:?}");
    }
}
