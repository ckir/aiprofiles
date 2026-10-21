//! End-to-end tests of `agent-profile completions <shell>` through the real binary (SP5d design §2.1).

mod support;

use std::process::Output;

use support::Root;

const COMMAND_WORDS: [&str; 13] = [
    "current",
    "resolve",
    "status",
    "link",
    "unlink",
    "create",
    "delete",
    "list",
    "agents",
    "profiles",
    "repositories",
    "doctor",
    "completions",
];

const SHELLS: [&str; 5] = ["bash", "elvish", "fish", "powershell", "zsh"];

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn contract_rows_exit_two_with_exact_messages() {
    let root = Root::new();
    let cases: &[(&[&str], &str)] = &[
        (
            &["completions"],
            "`completions` needs a shell: agent-profile completions <bash|elvish|fish|powershell|zsh>",
        ),
        (
            &["completions", "nope"],
            "unknown shell `nope` (shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`)",
        ),
        (
            &["completions", "Bash"],
            "unknown shell `Bash` (shells: `bash`, `elvish`, `fish`, `powershell`, `zsh`)",
        ),
        (&["completions", "bash", "zsh"], "`completions` takes one shell"),
        (&["completions", "bash", "--json"], "unknown option \"--json\" for `completions`"),
        (&["completions", "bash", "--repo=x"], "unknown option \"--repo\" for `completions`"),
    ];
    for (args, message) in cases {
        let output = root.agent_profile(*args).output().unwrap();
        assert_eq!(output.status.code(), Some(2), "{args:?}: {}", stderr(&output));
        assert!(output.stdout.is_empty(), "{args:?}: {}", stdout(&output));
        assert_eq!(stderr(&output), format!("agent-profile: error: {message}\n"), "{args:?}");
    }
}

#[test]
fn existing_messages_match_agents_word_for_word() {
    let root = Root::new();
    let pairs: &[(&[&str], &[&str])] = &[
        (&["completions", "bash", "--repo", "x"], &["agents", "--repo", "x"]),
        (&["completions", "bash", "--", "x"], &["agents", "--", "x"]),
        (&["completions", "bash", "--bogus"], &["agents", "--bogus"]),
        (&["fake", "completions"], &["fake", "agents"]),
        (&["fake", "completions", "bash"], &["fake", "agents"]),
    ];
    for (completions, agents) in pairs {
        let left = root.agent_profile(*completions).output().unwrap();
        let right = root.agent_profile(*agents).output().unwrap();
        assert_eq!(left.status.code(), Some(2), "{completions:?}: {}", stderr(&left));
        assert_eq!(right.status.code(), Some(2), "{agents:?}: {}", stderr(&right));
        assert_eq!(
            stderr(&left),
            stderr(&right).replace("agents", "completions"),
            "{completions:?} against {agents:?}"
        );
    }
}

#[test]
fn help_wins_before_any_arity_check() {
    let root = Root::new();
    let usage = "Usage: agent-profile completions <shell>\nPrint a shell completion script to stdout. \
                 <shell> is bash, elvish, fish, powershell or zsh.\n";
    let cases: &[&[&str]] = &[
        &["completions", "--help"],
        &["completions", "-h"],
        &["completions", "bash", "extra", "--help"],
        &["fake", "completions", "--help"],
    ];
    for args in cases {
        let output = root.agent_profile(*args).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{args:?}: {}", stderr(&output));
        assert!(output.stderr.is_empty(), "{args:?}: {}", stderr(&output));
        assert_eq!(stdout(&output), usage, "{args:?}");
    }
}

#[test]
fn each_shell_prints_a_script() {
    let root = Root::new();
    for shell in SHELLS {
        let output = root.agent_profile(["completions", shell]).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{shell}: {}", stderr(&output));
        assert!(output.stderr.is_empty(), "{shell}: {}", stderr(&output));
        let text = stdout(&output);
        assert!(!text.is_empty(), "{shell}: empty script");
        for id in agent_profile::adapter::known_agents() {
            assert!(text.contains(id), "{shell}: script lacks agent {id}");
        }
        for word in COMMAND_WORDS {
            assert!(text.contains(word), "{shell}: script lacks command word {word}");
        }
    }
}

#[test]
fn completions_needs_no_root() {
    let root = Root::new();
    let output = root
        .agent_profile(["completions", "zsh"])
        .env("AGENT_PROFILE_HOME", "relative")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let absent = root.path().join("absent");
    let output = root
        .agent_profile(["completions", "zsh"])
        .env("AGENT_PROFILE_HOME", &absent)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(!absent.exists(), "completions created {}", absent.display());

    let empty = Root::empty();
    let output = empty.agent_profile(["completions", "fish"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        std::fs::read_dir(empty.path()).unwrap().count(),
        0,
        "completions wrote into the root"
    );
}

#[cfg(target_os = "linux")]
#[test]
fn a_failed_stdout_write_exits_one() {
    let root = Root::new();
    let mut command = std::process::Command::new("sh");
    command.args([
        "-c",
        "\"$0\" completions bash > /dev/full",
        env!("CARGO_BIN_EXE_agent-profile"),
    ]);
    for name in support::FIXTURE_VARS.iter().chain(support::WRAPPER_VARS.iter()) {
        command.env_remove(name);
    }
    command.env("AGENT_PROFILE_HOME", root.path());
    command.current_dir(root.path());
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stderr(&output).contains("could not write to stdout"), "{}", stderr(&output));
}
