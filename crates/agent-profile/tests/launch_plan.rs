//! `LaunchPlan::command` pinned independently of any adapter (SP1 design §8.4; spec §34 "LaunchPlan").

mod support;

use std::path::PathBuf;

use agent_profile::launch::LaunchPlan;

fn plan(cwd: Option<PathBuf>) -> LaunchPlan {
    LaunchPlan {
        executable: PathBuf::from(env!("CARGO_BIN_EXE_fake-agent")),
        args: vec!["--".into(), "a b".into(), "--flag".into()],
        env: vec![
            // Cargo and nextest set both variables in every test process; the plan overrides one.
            ("CARGO_MANIFEST_DIR".into(), "from-plan".into()),
            ("FAKE_AGENT_ECHO_ENV".into(), "CARGO_MANIFEST_DIR,CARGO_PKG_NAME".into()),
        ],
        cwd,
    }
}

fn run(plan: &LaunchPlan) -> serde_json::Value {
    let mut command = plan.command();
    for name in support::FIXTURE_VARS {
        if name != "FAKE_AGENT_ECHO_ENV" {
            command.env_remove(name);
        }
    }
    let output = command.output().unwrap();
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    support::report(&output.stdout)
}

#[test]
fn command_applies_cwd_env_overrides_and_args() {
    assert!(
        std::env::var_os("CARGO_MANIFEST_DIR").is_some()
            && std::env::var_os("CARGO_PKG_NAME").is_some()
    );
    let dir = tempfile::tempdir().unwrap();
    let report = run(&plan(Some(dir.path().to_path_buf())));
    assert_eq!(report["argv"], serde_json::json!(["--", "a b", "--flag"]));
    assert_eq!(
        report["env"],
        serde_json::json!({ "CARGO_MANIFEST_DIR": "from-plan", "CARGO_PKG_NAME": env!("CARGO_PKG_NAME") })
    );
    let cwd = PathBuf::from(report["cwd"].as_str().unwrap());
    assert_eq!(cwd.canonicalize().unwrap(), dir.path().canonicalize().unwrap());
}

#[test]
fn command_without_cwd_inherits_the_working_directory() {
    let report = run(&plan(None));
    let cwd = PathBuf::from(report["cwd"].as_str().unwrap());
    assert_eq!(
        cwd.canonicalize().unwrap(),
        std::env::current_dir().unwrap().canonicalize().unwrap()
    );
}
