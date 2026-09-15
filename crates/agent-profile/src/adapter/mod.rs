//! Agent adapters: supported agents (spec §2), capability semantics (spec §3), argument
//! conflicts (spec §21) and evidence metadata (spec §28).
//!
//! SP1 has no adapter trait (SP2 designs it) and one test-only agent, `fake`, compiled only when debug
//! assertions are on, so release builds contain no agent at all (design D1).

pub mod metadata;

use std::ffi::{OsStr, OsString};
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::{AppRoot, Config};
use crate::error::{Error, Result};
use crate::exe::Origin;
use crate::launch::LaunchPlan;
use crate::name::{AgentId, ProfileName};
use crate::resolve::Resolution;

/// A launch plus what dry run reports but the launcher does not need.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedLaunch {
    pub plan: LaunchPlan,
    pub profile: ProfileName,
    pub profile_dir: PathBuf,
    pub profile_dir_exists: bool,
    pub executable_origin: Origin,
    pub mechanism: String,
    /// Override variables whose values must never be printed (spec §22).
    pub sensitive_env: Vec<OsString>,
}

/// The agents this build knows.
pub fn known_agents() -> Vec<&'static str> {
    #[cfg(debug_assertions)]
    let agents = vec!["fake"];
    #[cfg(not(debug_assertions))]
    let agents = Vec::new();
    agents
}

/// Builds the launch for an explicitly resolved profile (design §5.1 step 5).
pub fn plan(
    agent: &AgentId,
    resolution: &Resolution,
    root: &AppRoot,
    config: &Config,
    args: Vec<OsString>,
    path_var: Option<&OsStr>,
) -> Result<PlannedLaunch> {
    let profile =
        resolution.profile.clone().ok_or_else(|| Error::NoProfile { agent: agent.to_string() })?;
    match agent.as_str() {
        #[cfg(debug_assertions)]
        "fake" => fake::plan(profile, root, config, args, path_var),
        other => {
            // Release builds know no agent, so these inputs are unused there.
            let _ = (profile, root, config, args, path_var);
            Err(Error::UnknownAgent {
                agent: other.to_owned(),
                known: known_agents().into_iter().map(String::from).collect(),
                unknown_configured: Vec::new(),
            })
        }
    }
}

/// Refuses a profile whose name differs from an existing `profiles/` entry only in ASCII case (design §7.3).
#[cfg_attr(not(debug_assertions), allow(dead_code))]
fn check_case_twins(root: &AppRoot, profile: &ProfileName) -> Result<()> {
    let Ok(entries) = fs::read_dir(root.profiles_dir()) else {
        return Ok(());
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name != profile.as_str() && name.eq_ignore_ascii_case(profile.as_str()) {
            return Err(Error::ProfileCaseConflict {
                requested: profile.to_string(),
                existing: name.to_owned(),
            });
        }
    }
    Ok(())
}

/// Lazily creates the profile directory (spec §9, §9.1): idempotent, and verified to be a directory.
pub fn ensure_profile_dir(planned: &PlannedLaunch) -> Result<()> {
    let dir = &planned.profile_dir;
    let error = |source: io::Error| Error::ProfileDir { path: dir.clone(), source };
    fs::create_dir_all(dir).map_err(error)?;
    if fs::metadata(dir).map_err(error)?.is_dir() {
        Ok(())
    } else {
        Err(error(io::Error::other("the path exists but is not a directory")))
    }
}

#[cfg(debug_assertions)]
mod fake {
    use super::*;
    use crate::exe;

    pub(super) const HOME_VAR: &str = "FAKE_AGENT_HOME";

    pub(super) fn plan(
        profile: ProfileName,
        root: &AppRoot,
        config: &Config,
        args: Vec<OsString>,
        path_var: Option<&OsStr>,
    ) -> Result<PlannedLaunch> {
        let found = exe::discover(
            "fake",
            "fake-agent",
            config.agent_executable("fake"),
            path_var,
            &root.config_path(),
        )?;
        check_case_twins(root, &profile)?;
        let profile_dir = root.profiles_dir().join(profile.as_str()).join("fake");
        let profile_dir_exists = profile_dir.is_dir();
        Ok(PlannedLaunch {
            plan: LaunchPlan {
                executable: found.path,
                args,
                env: vec![(HOME_VAR.into(), profile_dir.clone().into_os_string())],
                cwd: None,
            },
            profile,
            profile_dir,
            profile_dir_exists,
            executable_origin: found.origin,
            mechanism: format!("environment variable {HOME_VAR}"),
            sensitive_env: Vec::new(),
        })
    }
}

#[cfg(all(test, debug_assertions))]
mod tests {
    use super::*;
    use crate::name::Platform;
    use crate::resolve::resolve;

    fn setup() -> (tempfile::TempDir, AppRoot, Config, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let root = AppRoot::from_path(dir.path().join("root"));
        let exe = dir.path().join("fake-agent-bin");
        fs::write(&exe, b"x").unwrap();
        fs::create_dir_all(root.path()).unwrap();
        fs::write(
            root.config_path(),
            format!("[agents.fake]\nexecutable = {:?}\n", exe.to_str().unwrap()),
        )
        .unwrap();
        let config = Config::load(&root).unwrap();
        (dir, root, config, exe)
    }

    fn resolution(profile: &str) -> Resolution {
        resolve(
            AgentId::parse("fake").unwrap(),
            Some(ProfileName::parse(profile, Platform::host()).unwrap()),
        )
    }

    #[test]
    fn fake_plan_sets_home_override_and_passes_args_verbatim() {
        let (_dir, root, config, exe) = setup();
        let args: Vec<OsString> = vec!["--foo".into(), "a b".into()];
        let planned = plan(
            &AgentId::parse("fake").unwrap(),
            &resolution("work"),
            &root,
            &config,
            args.clone(),
            None,
        )
        .unwrap();
        let dir = root.profiles_dir().join("work").join("fake");
        assert_eq!(
            planned.plan,
            LaunchPlan {
                executable: exe,
                args,
                env: vec![("FAKE_AGENT_HOME".into(), dir.clone().into_os_string())],
                cwd: None
            }
        );
        assert_eq!(planned.profile_dir, dir);
        assert!(!planned.profile_dir_exists);
        assert_eq!(planned.executable_origin, Origin::Configured);
    }

    #[test]
    fn no_profile_is_an_error() {
        let (_dir, root, config, _exe) = setup();
        let agent = AgentId::parse("fake").unwrap();
        let error =
            plan(&agent, &resolve(agent.clone(), None), &root, &config, vec![], None).unwrap_err();
        assert!(matches!(error, Error::NoProfile { .. }));
    }

    #[test]
    fn case_only_twin_is_refused_for_any_entry_type() {
        let (_dir, root, config, _exe) = setup();
        let agent = AgentId::parse("fake").unwrap();
        fs::create_dir_all(root.profiles_dir()).unwrap();
        fs::write(root.profiles_dir().join("work"), b"a file, not a directory").unwrap();
        let error = plan(&agent, &resolution("WORK"), &root, &config, vec![], None).unwrap_err();
        assert!(
            matches!(error, Error::ProfileCaseConflict { ref existing, .. } if existing == "work"),
            "{error:?}"
        );
        assert!(plan(&agent, &resolution("work"), &root, &config, vec![], None).is_ok());
    }

    #[test]
    fn ensure_profile_dir_is_idempotent_and_rejects_a_file() {
        let (_dir, root, config, _exe) = setup();
        let agent = AgentId::parse("fake").unwrap();
        let planned = plan(&agent, &resolution("work"), &root, &config, vec![], None).unwrap();
        ensure_profile_dir(&planned).unwrap();
        ensure_profile_dir(&planned).unwrap();
        assert!(planned.profile_dir.is_dir());

        let blocked = plan(&agent, &resolution("blocked"), &root, &config, vec![], None).unwrap();
        fs::create_dir_all(blocked.profile_dir.parent().unwrap()).unwrap();
        fs::write(&blocked.profile_dir, b"file").unwrap();
        assert!(matches!(ensure_profile_dir(&blocked), Err(Error::ProfileDir { .. })));
    }
}
