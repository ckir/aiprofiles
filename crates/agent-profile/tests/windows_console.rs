//! Windows control-event and job-object behaviour (spec §24; SP1 design §7.6, §8.4).
#![cfg(windows)]

mod support;

use std::io::{BufRead, BufReader};
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

use support::Root;
use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT};
use windows_sys::Win32::Storage::FileSystem::SYNCHRONIZE;
use windows_sys::Win32::System::Threading::{
    CREATE_NEW_CONSOLE, OpenProcess, PROCESS_TERMINATE, TerminateProcess, WaitForSingleObject,
};

/// `STATUS_CONTROL_C_EXIT`, the exit code of a process ended by default Ctrl-C or Ctrl-Break processing.
const STATUS_CONTROL_C_EXIT: i32 = 0xC000013A_u32 as i32;

/// Kills a child process if it is still running when dropped, so a failed assertion leaks nothing.
struct ChildGuard(Option<Child>);

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// An owned process handle that terminates the process on drop if it is still running.
struct ProcessGuard(HANDLE);

impl ProcessGuard {
    fn open(pid: u32) -> ProcessGuard {
        // SAFETY: plain Win32 call; a null result is checked.
        let handle = unsafe { OpenProcess(SYNCHRONIZE | PROCESS_TERMINATE, 0, pid) };
        assert!(
            !handle.is_null(),
            "cannot open process {pid}: {}",
            std::io::Error::last_os_error()
        );
        ProcessGuard(handle)
    }

    fn has_exited_within(&self, timeout: Duration) -> bool {
        // SAFETY: `self.0` is a valid process handle with SYNCHRONIZE access.
        let wait = unsafe { WaitForSingleObject(self.0, timeout.as_millis() as u32) };
        assert!(
            wait == WAIT_OBJECT_0 || wait == WAIT_TIMEOUT,
            "WaitForSingleObject returned {wait}"
        );
        wait == WAIT_OBJECT_0
    }
}

impl Drop for ProcessGuard {
    fn drop(&mut self) {
        // SAFETY: `self.0` is a valid handle owned by this guard.
        unsafe {
            if WaitForSingleObject(self.0, 0) != WAIT_OBJECT_0 {
                TerminateProcess(self.0, 1);
            }
            CloseHandle(self.0);
        }
    }
}

/// Runs `agent-profile fake work` under `console-driver` in a new console and returns its result.
fn drive(event: &str, readiness: &str, env: &[(&str, &str)]) -> serde_json::Value {
    let root = Root::new();
    let result = root.path().join("result.json");
    let mut command = Command::new(env!("CARGO_BIN_EXE_console-driver"));
    command
        .arg(&result)
        .arg(event)
        .arg(readiness)
        .arg(env!("CARGO_BIN_EXE_agent-profile"))
        .args(["fake", "work"]);
    for name in support::FIXTURE_VARS.iter().chain(support::WRAPPER_VARS.iter()) {
        command.env_remove(name);
    }
    command.env("AGENT_PROFILE_HOME", root.path());
    for (name, value) in env {
        command.env(name, value);
    }
    let status = command
        .creation_flags(CREATE_NEW_CONSOLE)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(0), "console-driver failed");
    let result: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&result).unwrap()).unwrap();
    assert!(result.get("error").is_none(), "{result}");
    result
}

fn exit_code(result: &serde_json::Value) -> i64 {
    result["exit_code"].as_i64().unwrap_or_else(|| panic!("no exit code: {result}"))
}

#[test]
fn ctrl_c_reaches_the_agent_and_the_wrapper_waits_for_it() {
    let result = drive(
        "ctrl-c",
        "report",
        &[("FAKE_AGENT_CTRL_C_EXIT", "42"), ("FAKE_AGENT_SLEEP_MS", "30000")],
    );
    assert_eq!(exit_code(&result), 42, "{result}");
}

#[test]
fn ctrl_break_reaches_the_agent_and_the_wrapper_waits_for_it() {
    let result = drive(
        "ctrl-break",
        "report",
        &[("FAKE_AGENT_CTRL_C_EXIT", "43"), ("FAKE_AGENT_SLEEP_MS", "30000")],
    );
    assert_eq!(exit_code(&result), 43, "{result}");
}

#[test]
fn ctrl_c_default_agent_exit_code_is_propagated() {
    let result = drive("ctrl-c", "report", &[("FAKE_AGENT_SLEEP_MS", "30000")]);
    assert_eq!(exit_code(&result), i64::from(STATUS_CONTROL_C_EXIT), "{result}");
}

#[test]
fn ctrl_c_between_handler_install_and_spawn_is_swallowed() {
    let result = drive(
        "ctrl-c",
        "pause-marker",
        &[("AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS", "2000"), ("FAKE_AGENT_EXIT", "7")],
    );
    assert_eq!(exit_code(&result), 7, "{result}");
    assert!(result["stdout"].as_str().unwrap().contains("\"pid\""), "{result}");
}

#[test]
fn normal_and_non_zero_exits_through_the_job_path() {
    assert_eq!(exit_code(&drive("none", "none", &[])), 0);
    assert_eq!(exit_code(&drive("none", "none", &[("FAKE_AGENT_EXIT", "3")])), 3);
}

/// Spawns the wrapper directly (no new console) with piped stdout and stderr.
fn spawn_wrapper(root: &Root, env: &[(&str, &str)]) -> ChildGuard {
    let mut command = root.agent_profile(["fake", "work"]);
    for (name, value) in env {
        command.env(name, value);
    }
    ChildGuard(Some(
        command.stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap(),
    ))
}

/// The first line of a pipe, read on a helper thread with a timeout.
fn first_line(pipe: impl std::io::Read + Send + 'static) -> String {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let mut line = String::new();
        let _ = BufReader::new(pipe).read_line(&mut line);
        let _ = tx.send(line);
    });
    rx.recv_timeout(Duration::from_secs(30)).expect("no line within 30 s")
}

#[test]
fn killing_the_wrapper_leaves_no_orphaned_agent() {
    let root = Root::new();
    let mut wrapper = spawn_wrapper(&root, &[("FAKE_AGENT_SLEEP_MS", "30000")]);
    let child = wrapper.0.as_mut().unwrap();
    let report = support::report(first_line(child.stdout.take().unwrap()).as_bytes());
    let agent = ProcessGuard::open(report["pid"].as_u64().unwrap() as u32);
    assert!(!agent.has_exited_within(Duration::ZERO));
    child.kill().unwrap();
    child.wait().unwrap();
    assert!(agent.has_exited_within(Duration::from_secs(10)), "the agent outlived its wrapper");
}

#[test]
fn processes_the_agent_left_running_survive_a_normal_exit() {
    let root = Root::new();
    // Read the report and wait for the wrapper, not for the pipes: on Windows the sleeper inherits every
    // inheritable handle, including the stdout pipe, so the pipes stay open until the sleeper exits.
    let mut wrapper = spawn_wrapper(&root, &[("FAKE_AGENT_SPAWN_SLEEPER", "20000")]);
    let child = wrapper.0.as_mut().unwrap();
    let report = support::report(first_line(child.stdout.take().unwrap()).as_bytes());
    let sleeper = ProcessGuard::open(report["sleeper_pid"].as_u64().unwrap() as u32);
    assert_eq!(child.wait().unwrap().code(), Some(0));
    std::thread::sleep(Duration::from_millis(500));
    assert!(!sleeper.has_exited_within(Duration::ZERO), "the job killed a background process");
}

#[test]
fn breakaway_process_creation_matches_direct_invocation() {
    // CREATE_BREAKAWAY_FROM_JOB fails when the creator's innermost job forbids breakaway. Whatever the test
    // runner's own job allows (cargo test's job forbids it, nextest's per-test job allows it, both measured), the
    // wrapper must not change the outcome (V3 §24): compare against a direct run.
    let env = [("FAKE_AGENT_SPAWN_SLEEPER", "1000"), ("FAKE_AGENT_BREAKAWAY", "1")];
    let mut direct = support::fake_agent();
    let root = Root::new();
    let mut wrapped = root.agent_profile(["fake", "work"]);
    for (name, value) in env {
        direct.env(name, value);
        wrapped.env(name, value);
    }
    let direct = direct.output().unwrap();
    let wrapped = wrapped.output().unwrap();
    assert_eq!(
        wrapped.status.code(),
        direct.status.code(),
        "direct stderr: {}\nwrapped stderr: {}",
        String::from_utf8_lossy(&direct.stderr),
        String::from_utf8_lossy(&wrapped.stderr)
    );
}

#[test]
fn killing_the_wrapper_before_spawn_starts_no_agent() {
    let root = Root::new();
    let mut wrapper =
        spawn_wrapper(&root, &[("AGENT_PROFILE_DEBUG_PAUSE_BEFORE_SPAWN_MS", "10000")]);
    let child = wrapper.0.as_mut().unwrap();
    let marker = first_line(child.stderr.take().unwrap());
    assert_eq!(marker.trim_end(), "agent-profile: debug: paused before spawn");
    child.kill().unwrap();
    child.wait().unwrap();
    let mut stdout = String::new();
    std::io::Read::read_to_string(&mut child.stdout.take().unwrap(), &mut stdout).unwrap();
    assert_eq!(stdout, "", "an agent started after the wrapper was killed");
}
