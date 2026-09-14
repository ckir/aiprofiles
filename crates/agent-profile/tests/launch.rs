//! End-to-end launch tests through the real `agent-profile` binary (SP1 design §8.4; spec §34
//! "Passthrough", "Environment", "Process behavior").

mod support;

use std::io::Write;
use std::process::{Output, Stdio};

use support::Root;

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn passthrough_arguments_arrive_verbatim() {
    let root = Root::new();
    let output =
        root.agent_profile(["fake", "work", "--", "--foo", "bar", "a b"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(support::report(&output.stdout)["argv"], serde_json::json!(["--foo", "bar", "a b"]));

    let output =
        root.agent_profile(["fake", "work", "--", "--dry-run", "--", "--help"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        support::report(&output.stdout)["argv"],
        serde_json::json!(["--dry-run", "--", "--help"])
    );
}

#[test]
fn non_utf8_opaque_argument_arrives_unconverted() {
    let root = Root::new();
    let mut command = root.agent_profile(["fake", "work", "--"]);
    command.arg(support::non_utf8());
    let output = command.output().unwrap();
    // The fixture rejects a non-UTF-8 argument (exit 125), which proves it arrived without lossy conversion.
    assert_eq!(output.status.code(), Some(125), "{}", stderr(&output));
    assert!(stderr(&output).contains("argument is not valid UTF-8"), "{}", stderr(&output));
}

#[test]
fn profile_override_replaces_the_inherited_value() {
    let root = Root::new();
    let before = std::env::var_os("FAKE_AGENT_HOME");
    let output = root
        .agent_profile(["fake", "work"])
        .env("FAKE_AGENT_HOME", "/wrong")
        .env("FAKE_AGENT_ECHO_ENV", "FAKE_AGENT_HOME")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        support::report(&output.stdout)["env"]["FAKE_AGENT_HOME"],
        serde_json::json!(root.profile_dir("work").to_str().unwrap())
    );
    assert_eq!(std::env::var_os("FAKE_AGENT_HOME"), before);
}

#[test]
fn agent_exit_status_is_the_wrapper_exit_status() {
    let root = Root::new();
    let output = root.agent_profile(["fake", "work"]).env("FAKE_AGENT_EXIT", "7").output().unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert!(support::report(&output.stdout).get("pid").is_some());
}

#[test]
fn dry_run_reports_every_field_and_launches_nothing() {
    let root = Root::new();
    let output = root.agent_profile(["fake", "work", "--dry-run", "--", "--foo"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    for prefix in [
        "agent:",
        "profile:",
        "executable:",
        "repository:",
        "mechanism:",
        "environment:",
        "arguments:",
    ] {
        assert_eq!(
            text.lines().filter(|line| line.starts_with(prefix)).count(),
            1,
            "{prefix}\n{text}"
        );
    }
    assert!(text.contains("work (explicit)"), "{text}");
    assert!(text.contains("(would be created)"), "{text}");
    assert!(text.contains("[\"--foo\"]"), "{text}");
    assert!(!text.contains("\"pid\""), "the agent must not run: {text}");
    assert!(!root.profile_dir("work").exists());
}

#[test]
fn dry_run_of_an_agent_that_is_not_installed_fails_like_a_launch() {
    let root = Root::empty();
    let output =
        root.agent_profile(["fake", "work", "--dry-run"]).env("PATH", "").output().unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(output.stdout.is_empty());
}

#[test]
fn verbose_launch_reports_to_stderr_after_creating_the_profile_directory() {
    let root = Root::new();
    let output = root.agent_profile(["fake", "work", "--verbose"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stderr(&output);
    let report: Vec<&str> =
        text.lines().filter(|line| line.starts_with("agent-profile: ")).collect();
    assert_eq!(report.len(), 7, "{text}");
    assert!(report[0].starts_with("agent-profile: agent:"), "{text}");
    assert!(text.contains("agent-profile: environment:  FAKE_AGENT_HOME="), "{text}");
    assert!(!text.contains("(would be created)"), "{text}");
    assert!(support::report(&output.stdout).get("pid").is_some());
}

#[test]
fn dry_run_of_an_existing_profile_omits_would_be_created() {
    let root = Root::new();
    let first = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(first.status.code(), Some(0), "{}", stderr(&first));
    assert!(root.profile_dir("work").is_dir());
    let output = root.agent_profile(["fake", "work", "--dry-run"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = stdout(&output);
    let environment: Vec<&str> =
        text.lines().filter(|line| line.starts_with("environment:")).collect();
    assert_eq!(environment.len(), 1, "{text}");
    assert!(environment[0].contains("FAKE_AGENT_HOME="), "{text}");
    assert!(!text.contains("(would be created)"), "{text}");
}

#[test]
fn launch_creates_the_profile_directory_lazily() {
    let root = Root::new();
    assert!(!root.profile_dir("work").exists());
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn a_file_at_the_profile_path_is_a_profile_error_and_nothing_launches() {
    let root = Root::new();
    let dir = root.profile_dir("work");
    std::fs::create_dir_all(dir.parent().unwrap()).unwrap();
    std::fs::write(&dir, b"not a directory").unwrap();
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(output.stdout.is_empty());
}

#[test]
fn concurrent_first_launches_of_one_profile_all_succeed() {
    let root = Root::new();
    let children: Vec<_> = (0..8)
        .map(|_| root.agent_profile(["fake", "work"]).stdout(Stdio::piped()).spawn().unwrap())
        .collect();
    for child in children {
        let output = child.wait_with_output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    }
    assert!(root.profile_dir("work").is_dir());
}

#[test]
fn stdin_and_stderr_are_inherited() {
    let root = Root::new();
    let mut child = root
        .agent_profile(["fake", "work"])
        .env("FAKE_AGENT_STDIN", "1")
        .env("FAKE_AGENT_STDERR", "agent stderr")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"typed input").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(support::report(&output.stdout)["stdin"], serde_json::json!("typed input"));
    assert_eq!(stderr(&output), "agent stderr");
}

#[test]
fn help_and_version_after_the_agent_word() {
    let root = Root::new();
    for args in [&["fake", "--help"][..], &["zzz", "-h"], &["fake", "work", "--help", "--", "x"]] {
        let output = root.agent_profile(args).output().unwrap();
        assert_eq!(output.status.code(), Some(0), "{args:?}");
        assert!(stdout(&output).contains("Usage: agent-profile <agent> <profile>"), "{args:?}");
    }
    let output = root.agent_profile(["fake", "work", "-V"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(stdout(&output), format!("agent-profile {}\n", env!("CARGO_PKG_VERSION")));
}

#[test]
fn behaviour_table_rows_with_non_zero_exits() {
    let root = Root::new();
    let cases: &[(&[&str], i32, &str)] = &[
        (&["doctor"], 2, "`doctor` is not yet implemented"),
        (&["link", "work", "extra"], 2, "`link` is not yet implemented"),
        (&["fake", "create", "work"], 2, "`fake create` is not yet implemented"),
        (&["fake", "work", "--json"], 2, "`--json` is not yet implemented"),
        (&["zzz", "work"], 2, "unknown agent `zzz` (known agents: `fake`)"),
        (&["Fake", "work"], 2, "unknown agent `Fake`"),
        (&["fake", "work", "--bogus"], 2, "unknown option"),
        (&["fake", "work", "extra"], 2, "agent arguments must follow `--`"),
        (&["fake"], 4, "no profile selected for `fake`"),
        (&["fake", ".hidden"], 4, "invalid profile name"),
        (&["fake", ""], 4, "must not be empty"),
    ];
    for (args, code, message) in cases {
        let output = root.agent_profile(*args).output().unwrap();
        assert_eq!(output.status.code(), Some(*code), "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains(message), "{args:?}: {}", stderr(&output));
        assert!(output.stdout.is_empty(), "{args:?}");
    }
}

#[test]
fn bare_invocation_prints_top_level_help_to_stderr_only() {
    let root = Root::new();
    let output = root.agent_profile(Vec::<&str>::new()).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "{}", stdout(&output));
    assert!(
        stderr(&output).contains("Usage: agent-profile <agent> <profile>"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn explicit_executable_missing_is_not_installed() {
    let root = Root::empty();
    let missing = root.path().join("missing-agent");
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", missing.to_str().unwrap()));
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    assert!(stderr(&output).contains("does not exist or is not a file"), "{}", stderr(&output));
}

#[test]
fn not_on_path_is_not_installed_and_lists_unknown_configured_agents() {
    let root = Root::empty();
    root.write_config("[agents.fakr]\n");
    let output = root.agent_profile(["fake", "work"]).env("PATH", "").output().unwrap();
    assert_eq!(output.status.code(), Some(3), "{}", stderr(&output));
    let text = stderr(&output);
    assert!(text.contains("not found in PATH"), "{text}");
    assert!(text.contains("config.toml also configures unknown agents: `fakr`"), "{text}");
}

#[test]
fn unknown_agent_lists_unknown_configured_agents() {
    let root = Root::new();
    root.write_config("[agents.fakr]\n");
    let output = root.agent_profile(["fakr", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("config.toml also configures unknown agents: `fakr`"));
}

#[test]
fn corrupt_configuration_is_an_error_and_stays_untouched() {
    let root = Root::empty();
    let corrupt = "[agents.fake\nexecutable = ";
    root.write_config(corrupt);
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    assert!(
        stderr(&output).contains("never rewrites an invalid configuration"),
        "{}",
        stderr(&output)
    );
    assert_eq!(std::fs::read_to_string(root.path().join("config.toml")).unwrap(), corrupt);
}

#[test]
fn relative_app_root_is_an_error() {
    let root = Root::new();
    let output = root
        .agent_profile(["fake", "work"])
        .env("AGENT_PROFILE_HOME", "relative")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
}

#[test]
fn batch_file_override_is_refused() {
    let root = Root::empty();
    let batch = root.path().join("agent.cmd");
    std::fs::write(&batch, b"@echo off\r\n").unwrap();
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", batch.to_str().unwrap()));
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(6), "{}", stderr(&output));
    assert!(stderr(&output).contains("never through cmd.exe"), "{}", stderr(&output));
}

#[test]
fn garbage_executable_is_a_launch_failure() {
    let root = Root::empty();
    let name = if cfg!(windows) { "garbage.exe" } else { "garbage" };
    let garbage = root.path().join(name);
    std::fs::write(&garbage, b"this is not a program").unwrap();
    root.write_config(&format!("[agents.fake]\nexecutable = {:?}\n", garbage.to_str().unwrap()));
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(6), "{}", stderr(&output));
    assert!(stderr(&output).contains("could not launch"), "{}", stderr(&output));
}

#[test]
fn case_only_twin_is_refused_on_every_os() {
    let root = Root::new();
    std::fs::create_dir_all(root.path().join("profiles").join("work")).unwrap();
    for args in [&["fake", "WORK"][..], &["fake", "WORK", "--dry-run"]] {
        let output = root.agent_profile(args).output().unwrap();
        assert_eq!(output.status.code(), Some(4), "{args:?}: {}", stderr(&output));
        assert!(stderr(&output).contains("existing profile entry `work`"), "{}", stderr(&output));
        assert!(output.stdout.is_empty());
    }
    let names: Vec<String> = std::fs::read_dir(root.path().join("profiles"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["work"]);
    let output = root.agent_profile(["fake", "work"]).output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[cfg(unix)]
#[test]
fn unix_launch_replaces_the_wrapper_process() {
    let root = Root::new();
    let child = root.agent_profile(["fake", "work"]).stdout(Stdio::piped()).spawn().unwrap();
    let wrapper_pid = child.id();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(support::report(&output.stdout)["pid"], serde_json::json!(wrapper_pid));
}
